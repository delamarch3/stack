use std::cmp::Ordering;
use std::io::{Cursor, Read};
use std::mem;

use crate::stack::OperandStack;
use crate::Number;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum Bytecode {
    Push = 0,
    Pop,
    Load,
    Store,
    Add,
    Sub,
    Mul,
    Div,
    Cmp,
    Jmp,
    JmpLt,
    JmpEq,
    JmpGt,
    JmpNe,
    Swap,
    Dup,
    Over,
    Rot,

    Call,
    Fail,
    Ret,
}

macro_rules! slot {
    ($ty:ty, $i:expr) => {{
        let from = $i * mem::size_of::<$ty>();
        let to = from + mem::size_of::<$ty>();
        from..to
    }};
}

macro_rules! next {
    ($pc:expr, $ty:ty) => {{
        const N: usize = size_of::<$ty>();
        let buf = $pc.next::<N>()?;
        let val = <$ty>::from_le_bytes(buf);
        val
    }};
}

#[derive(Clone)]
pub struct ProgramCounter<'a> {
    src: Cursor<&'a [u8]>,
}

impl<'a> ProgramCounter<'a> {
    pub fn new(src: &'a [u8]) -> Self {
        let src = Cursor::new(src);
        Self { src }
    }

    fn set(&mut self, position: u64) {
        self.src.set_position(position);
    }

    fn position(&self) -> u64 {
        self.src.position()
    }

    fn next<const N: usize>(&mut self) -> Result<[u8; N]> {
        let mut buf = [0u8; N];
        let n = self.src.read(&mut buf)?;
        if n < N {
            assert_eq!(n, 0);
            Err(format!("read less than expected bytes: {n}"))?;
        }

        Ok(buf)
    }

    fn next_op(&mut self) -> Result<Bytecode> {
        const N: usize = size_of::<u8>();
        let buf = self.next::<N>()?;
        let op = u8::from_le_bytes(buf);
        assert!(op <= Bytecode::Ret as u8);
        let op = unsafe { std::mem::transmute::<_, Bytecode>(op) };
        Ok(op)
    }

    fn next_i32(&mut self) -> Result<i32> {
        Ok(next!(self, i32))
    }

    fn next_usize(&mut self) -> Result<usize> {
        Ok(next!(self, usize))
    }
}

const LOCALS_SIZE: usize = mem::size_of::<i32>() * 128;
pub struct Locals {
    locals: [u8; LOCALS_SIZE],
}

impl Default for Locals {
    fn default() -> Self {
        Self {
            locals: [0u8; LOCALS_SIZE],
        }
    }
}

impl Locals {
    fn read<T: Number>(&self, i: usize) -> T {
        T::from_le_bytes(&self.locals[slot!(T, i)])
    }

    fn write<T: Number>(&mut self, i: usize, value: T) {
        self.locals[slot!(T, i)].copy_from_slice(value.to_le_bytes().as_ref());
    }
}

const ENTRY_RET: usize = 0;
pub struct Frame {
    opstack: OperandStack,
    locals: Locals,
    entry: usize,
    ret: usize,
}

pub enum FrameResult {
    Call(Frame),
    Ret,
}

impl Frame {
    pub fn new(locals: Locals, opstack: OperandStack, entry: usize, ret: usize) -> Self {
        Self {
            opstack,
            locals,
            entry,
            ret,
        }
    }

    pub fn run<'a>(&mut self, pc: &mut ProgramCounter<'_>) -> Result<FrameResult> {
        loop {
            match pc.next_op()? {
                Bytecode::Push => {
                    let val = pc.next_i32()?;
                    self.opstack.push(val);
                }
                Bytecode::Pop => {
                    self.opstack.pop();
                }
                Bytecode::Load => {
                    let i = pc.next_usize()?;
                    let val = self.locals.read::<i32>(i);
                    self.opstack.push(val);
                }
                Bytecode::Store => {
                    let i = pc.next_usize()?;
                    let val = self.opstack.pop();
                    self.locals.write::<i32>(i, val);
                }
                Bytecode::Add => self.opstack.add(),
                Bytecode::Sub => self.opstack.sub(),
                Bytecode::Mul => self.opstack.mul(),
                Bytecode::Div => self.opstack.div(),
                Bytecode::Cmp => {
                    let lhs = pc.next_i32()?;
                    self.opstack.cmp(lhs);
                }
                Bytecode::Jmp => {
                    let pos = pc.next_usize()?;
                    pc.set(pos as u64);
                }
                Bytecode::JmpEq => {
                    let pos = pc.next_usize()?;
                    if self.opstack.pop() == Ordering::Equal as i32 {
                        pc.set(pos as u64);
                    }
                }
                Bytecode::JmpNe => {
                    let pos = pc.next_usize()?;
                    if self.opstack.pop() != Ordering::Equal as i32 {
                        pc.set(pos as u64);
                    }
                }
                Bytecode::JmpLt => {
                    let pos = pc.next_usize()?;
                    if self.opstack.pop() == Ordering::Less as i32 {
                        pc.set(pos as u64);
                    }
                }
                Bytecode::JmpGt => {
                    let pos = pc.next_usize()?;
                    if self.opstack.pop() == Ordering::Greater as i32 {
                        pc.set(pos as u64);
                    }
                }
                Bytecode::Swap => self.opstack.swap(),
                Bytecode::Dup => self.opstack.dup(),
                Bytecode::Over => self.opstack.over(),
                Bytecode::Rot => self.opstack.rot(),
                Bytecode::Call => {
                    let entry = pc.next_usize()?;
                    let ret = pc.position() as usize;
                    let opstack = OperandStack::default();
                    let mut locals = Locals::default();
                    (0..self.opstack.size())
                        .rev()
                        .for_each(|i| locals.write::<i32>(i, self.opstack.pop()));
                    let frame = Frame::new(locals, opstack, entry, ret);
                    break Ok(FrameResult::Call(frame));
                }
                Bytecode::Fail => Err("FAILED")?,
                Bytecode::Ret => break Ok(FrameResult::Ret),
            }
        }
    }
}

pub struct Interpreter<'a> {
    pc: ProgramCounter<'a>,
    frames: Vec<Frame>,
}

impl<'a> Interpreter<'a> {
    pub fn new(program: &'a [u8]) -> Result<Self> {
        let mut pc = ProgramCounter::new(program);
        let entry = pc.next_usize()?;
        let opstack = OperandStack::default();
        let locals = Locals::default();
        let main = Frame::new(locals, opstack, entry, ENTRY_RET);
        let frames = vec![main];

        Ok(Self { pc, frames })
    }

    pub fn opstack(&self) -> &OperandStack {
        &self.frames[0].opstack
    }

    pub fn run(&mut self) -> Result<()> {
        while let Some(mut current) = self.frames.pop() {
            let len = self.frames.len();
            let main = len == 0;

            match current.run(&mut self.pc)? {
                FrameResult::Call(next) => {
                    self.pc.set(next.entry as u64);
                    self.frames.push(current);
                    self.frames.push(next);
                }
                FrameResult::Ret if main => {
                    self.frames.push(current);
                    break;
                }
                FrameResult::Ret => {
                    self.pc.set(current.ret as u64);
                    // TODO: should functions return values by popping onto caller frame?
                    self.frames[len - 1].opstack.push(current.opstack.pop());
                }
            }
        }
        Ok(())
    }
}
