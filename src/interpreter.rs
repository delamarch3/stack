use std::cmp::Ordering;
use std::io::{Cursor, Read};
use std::mem;

use crate::stack::OperandStack;
use crate::Number;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Bytecode {
    Push,
    PushD,
    PushB,
    Pop,
    PopD,
    PopB,
    Load,
    LoadD,
    LoadB,
    Store,
    StoreD,
    StoreB,
    Get,
    GetD,
    GetB,
    Add,
    AddD,
    AddB,
    Sub,
    SubD,
    SubB,
    Mul,
    MulD,
    Div,
    DivD,
    Cmp,
    CmpD,
    Dup,
    DupD,

    Jmp,
    JmpLt,
    JmpEq,
    JmpGt,
    JmpNe,
    Call,
    Fail,
    Ret,
    RetW,
    RetD,
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
pub struct Program<'a> {
    src: &'a [u8],
    counter: Cursor<&'a [u8]>,
}

impl<'a> Program<'a> {
    pub fn new(src: &'a [u8]) -> Self {
        let counter = Cursor::new(src);
        Self { src, counter }
    }

    fn set(&mut self, position: u64) {
        self.counter.set_position(position);
    }

    fn position(&self) -> u64 {
        self.counter.position()
    }

    fn next<const N: usize>(&mut self) -> Result<[u8; N]> {
        let mut buf = [0u8; N];
        let n = self.counter.read(&mut buf)?;
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
        assert!(op <= Bytecode::RetD as u8);
        let op = unsafe { std::mem::transmute::<_, Bytecode>(op) };
        Ok(op)
    }

    fn next_i8(&mut self) -> Result<i8> {
        Ok(next!(self, i8))
    }

    fn next_i32(&mut self) -> Result<i32> {
        Ok(next!(self, i32))
    }

    fn next_i64(&mut self) -> Result<i64> {
        Ok(next!(self, i64))
    }

    fn next_u64(&mut self) -> Result<u64> {
        Ok(next!(self, u64))
    }

    fn get<T: Number>(&mut self, offset: usize) -> T {
        T::from_le_bytes(&self.src[offset..offset + T::SIZE])
    }
}

const SLOT_SIZE: usize = std::mem::size_of::<i32>();
macro_rules! slot {
    ($ty:ty, $i:expr) => {{
        let from = $i * SLOT_SIZE;
        let to = from + mem::size_of::<$ty>();
        from..to
    }};
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
    fn read<T: Number>(&self, i: u64) -> T {
        T::from_le_bytes(&self.locals[slot!(T, i as usize)])
    }

    fn write<T: Number>(&mut self, i: u64, value: T) {
        self.locals[slot!(T, i as usize)].copy_from_slice(value.to_le_bytes().as_ref());
    }

    fn copy_from_slice(&mut self, slice: &[u8]) {
        self.locals[..slice.len()].copy_from_slice(slice);
    }
}

pub(crate) struct Frame {
    opstack: OperandStack,
    locals: Locals,
    entry: u64,
    ret: usize,
}

pub(crate) enum FrameResult {
    Call(Frame),
    Ret,
    RetW,
    RetD,
    Fail,
}

impl Frame {
    pub fn new(locals: Locals, opstack: OperandStack, entry: u64, ret: usize) -> Self {
        Self {
            opstack,
            locals,
            entry,
            ret,
        }
    }

    pub fn run<'a>(&mut self, pc: &mut Program<'_>) -> Result<FrameResult> {
        loop {
            match pc.next_op()? {
                Bytecode::Push => {
                    let val = pc.next_i32()?;
                    self.opstack.push(val);
                }
                Bytecode::PushD => {
                    let val = pc.next_i64()?;
                    self.opstack.push(val);
                }
                Bytecode::PushB => {
                    let val = pc.next_i8()?;
                    self.opstack.push(val);
                }
                Bytecode::Pop => {
                    self.opstack.pop::<i32>();
                }
                Bytecode::PopD => {
                    self.opstack.pop::<i64>();
                }
                Bytecode::PopB => {
                    self.opstack.pop::<i8>();
                }
                Bytecode::Load => {
                    let i = pc.next_u64()?;
                    let val = self.locals.read::<i32>(i);
                    self.opstack.push(val);
                }
                Bytecode::LoadD => {
                    let i = pc.next_u64()?;
                    let val = self.locals.read::<i64>(i);
                    self.opstack.push(val);
                }
                Bytecode::LoadB => {
                    let i = pc.next_u64()?;
                    let val = self.locals.read::<i8>(i);
                    self.opstack.push(val);
                }
                Bytecode::Store => {
                    let i = pc.next_u64()?;
                    let val = self.opstack.pop();
                    self.locals.write::<i32>(i, val);
                }
                Bytecode::StoreD => {
                    let i = pc.next_u64()?;
                    let val = self.opstack.pop();
                    self.locals.write::<i64>(i, val);
                }
                Bytecode::StoreB => {
                    let i = pc.next_u64()?;
                    let val = self.opstack.pop();
                    self.locals.write::<i8>(i, val);
                }
                Bytecode::Get => {
                    let ptr = self.opstack.pop::<u64>();
                    let offset = pc.next_u64()?;
                    let value = pc.get::<i32>((ptr + offset) as usize);
                    self.opstack.push(value);
                }
                Bytecode::GetD => {
                    let ptr = self.opstack.pop::<u64>();
                    let offset = pc.next_u64()?;
                    let value = pc.get::<i64>((ptr + offset) as usize);
                    self.opstack.push(value);
                }
                Bytecode::GetB => {
                    let ptr = self.opstack.pop::<u64>();
                    let offset = pc.next_u64()?;
                    let value = pc.get::<i8>((ptr + offset) as usize);
                    self.opstack.push(value);
                }
                Bytecode::Add => self.opstack.add::<i32>(),
                Bytecode::AddD => self.opstack.add::<i64>(),
                Bytecode::AddB => self.opstack.add::<i8>(),
                Bytecode::Sub => self.opstack.sub::<i32>(),
                Bytecode::SubD => self.opstack.sub::<i64>(),
                Bytecode::SubB => self.opstack.sub::<i8>(),
                Bytecode::Mul => self.opstack.mul::<i32>(),
                Bytecode::MulD => self.opstack.mul::<i64>(),
                Bytecode::Div => self.opstack.div::<i32>(),
                Bytecode::DivD => self.opstack.div::<i64>(),
                Bytecode::Cmp => self.opstack.cmp::<i32>(),
                Bytecode::CmpD => self.opstack.cmp::<i64>(),
                Bytecode::Jmp => {
                    let pos = pc.next_u64()?;
                    pc.set(pos as u64);
                }
                Bytecode::JmpEq => {
                    let pos = pc.next_u64()?;
                    if self.opstack.pop::<i32>() == Ordering::Equal as i32 {
                        pc.set(pos as u64);
                    }
                }
                Bytecode::JmpNe => {
                    let pos = pc.next_u64()?;
                    if self.opstack.pop::<i32>() != Ordering::Equal as i32 {
                        pc.set(pos as u64);
                    }
                }
                Bytecode::JmpLt => {
                    let pos = pc.next_u64()?;
                    if self.opstack.pop::<i32>() == Ordering::Less as i32 {
                        pc.set(pos as u64);
                    }
                }
                Bytecode::JmpGt => {
                    let pos = pc.next_u64()?;
                    if self.opstack.pop::<i32>() == Ordering::Greater as i32 {
                        pc.set(pos as u64);
                    }
                }
                Bytecode::Dup => self.opstack.dup::<i32>(),
                Bytecode::DupD => self.opstack.dup::<i64>(),
                Bytecode::Call => {
                    let mut locals = Locals::default();
                    locals.copy_from_slice(self.opstack.as_slice());
                    self.opstack.clear();
                    let entry = pc.next_u64()?;
                    let ret = pc.position() as usize;
                    let opstack = OperandStack::default();
                    let frame = Frame::new(locals, opstack, entry, ret);
                    break Ok(FrameResult::Call(frame));
                }
                Bytecode::Fail => break Ok(FrameResult::Fail),
                Bytecode::Ret => break Ok(FrameResult::Ret),
                Bytecode::RetW => break Ok(FrameResult::RetW),
                Bytecode::RetD => break Ok(FrameResult::RetD),
            }
        }
    }
}

pub struct Interpreter<'a> {
    pc: Program<'a>,
    frames: Vec<Frame>,
}

impl<'a> Interpreter<'a> {
    pub fn new(program: &'a [u8]) -> Result<Self> {
        let mut pc = Program::new(program);
        let entry = pc.next_u64()?;
        pc.set(entry);
        let opstack = OperandStack::default();
        let locals = Locals::default();
        let ret = 0;
        let main = Frame::new(locals, opstack, entry, ret);
        let frames = vec![main];

        Ok(Self { pc, frames })
    }

    pub fn print_opstack(&self) {
        println!("{}", self.frames[0].opstack)
    }

    pub fn run(&mut self) -> Result<()> {
        while let Some(mut current) = self.frames.pop() {
            let len = self.frames.len();
            let is_entry = len == 0;

            match current.run(&mut self.pc)? {
                FrameResult::Call(next) => {
                    self.pc.set(next.entry as u64);
                    self.frames.push(current);
                    self.frames.push(next);
                }
                FrameResult::Ret | FrameResult::RetW | FrameResult::RetD if is_entry => {
                    self.frames.push(current);
                    break;
                }
                FrameResult::Ret => {
                    self.pc.set(current.ret as u64);
                }
                FrameResult::RetW => {
                    self.pc.set(current.ret as u64);
                    self.frames[len - 1]
                        .opstack
                        .push::<i32>(current.opstack.pop());
                }
                FrameResult::RetD => {
                    self.pc.set(current.ret as u64);
                    self.frames[len - 1]
                        .opstack
                        .push::<i64>(current.opstack.pop());
                }
                FrameResult::Fail => Err("FAILED")?,
            }
        }

        Ok(())
    }
}
