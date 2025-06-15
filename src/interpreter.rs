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

    fn set_position(&mut self, position: u64) {
        self.counter.set_position(position);
    }

    fn position(&self) -> u64 {
        self.counter.position()
    }

    fn next<T: Number>(&mut self) -> Result<T> {
        let mut buf = [0u8; 8];
        let n = self.counter.read(&mut buf[0..T::SIZE])?;
        if n < T::SIZE {
            Err(format!("read less than expected bytes: {n}"))?;
        }

        Ok(T::from_le_bytes(&buf[0..T::SIZE]))
    }

    fn next_op(&mut self) -> Result<Bytecode> {
        let op = self.next::<u8>()?;
        assert!(op <= Bytecode::RetD as u8);
        let op = unsafe { std::mem::transmute::<_, Bytecode>(op) };
        Ok(op)
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
            if let Some(fr) = self.step(pc)? {
                return Ok(fr);
            }
        }
    }

    fn step(&mut self, pc: &mut Program) -> Result<Option<FrameResult>> {
        match pc.next_op()? {
            Bytecode::Push => self.push::<i32>(pc)?,
            Bytecode::PushD => self.push::<i64>(pc)?,
            Bytecode::PushB => self.push::<i8>(pc)?,
            Bytecode::Pop => self.opstack.drop::<i32>(),
            Bytecode::PopD => self.opstack.drop::<i64>(),
            Bytecode::PopB => self.opstack.drop::<i8>(),
            Bytecode::Load => self.load::<i32>(pc)?,
            Bytecode::LoadD => self.load::<i64>(pc)?,
            Bytecode::LoadB => self.load::<i8>(pc)?,
            Bytecode::Store => self.store::<i32>(pc)?,
            Bytecode::StoreD => self.store::<i64>(pc)?,
            Bytecode::StoreB => self.store::<i8>(pc)?,
            Bytecode::Get => self.get::<i32>(pc),
            Bytecode::GetD => self.get::<i64>(pc),
            Bytecode::GetB => self.get::<i8>(pc),
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
            Bytecode::Jmp => self.jmp(pc, &[])?,
            Bytecode::JmpEq => self.jmp(pc, &[Ordering::Equal])?,
            Bytecode::JmpNe => self.jmp(pc, &[Ordering::Greater, Ordering::Less])?,
            Bytecode::JmpLt => self.jmp(pc, &[Ordering::Less])?,
            Bytecode::JmpGt => self.jmp(pc, &[Ordering::Greater])?,
            Bytecode::Dup => self.opstack.dup::<i32>(),
            Bytecode::DupD => self.opstack.dup::<i64>(),
            Bytecode::Call => return self.call(pc).map(Some),
            Bytecode::Fail => return Ok(Some(FrameResult::Fail)),
            Bytecode::Ret => return Ok(Some(FrameResult::Ret)),
            Bytecode::RetW => return Ok(Some(FrameResult::RetW)),
            Bytecode::RetD => return Ok(Some(FrameResult::RetD)),
        }

        Ok(None)
    }

    fn push<T: Number>(&mut self, pc: &mut Program) -> Result<()> {
        let val = pc.next::<T>()?;
        self.opstack.push(val);
        Ok(())
    }

    fn load<T: Number>(&mut self, pc: &mut Program) -> Result<()> {
        let i = pc.next::<u64>()?;
        let val = self.locals.read::<T>(i);
        self.opstack.push(val);
        Ok(())
    }

    fn store<T: Number>(&mut self, pc: &mut Program) -> Result<()> {
        let i = pc.next::<u64>()?;
        let val = self.opstack.pop();
        self.locals.write::<T>(i, val);
        Ok(())
    }

    fn get<T: Number>(&mut self, pc: &mut Program) {
        let offset = self.opstack.pop::<u64>();
        let ptr = self.opstack.pop::<u64>();
        let value = pc.get::<T>((ptr + offset) as usize);
        self.opstack.push(value);
    }

    fn jmp(&mut self, pc: &mut Program, conditions: &[Ordering]) -> Result<()> {
        let pos = pc.next::<u64>()?;

        let jmp = match conditions {
            [] => true,
            cs => {
                let have = self.opstack.pop::<i32>();
                cs.iter().find(|&&want| want as i32 == have).is_some()
            }
        };

        if jmp {
            pc.set_position(pos);
        }

        Ok(())
    }

    fn call(&mut self, pc: &mut Program) -> Result<FrameResult> {
        let mut locals = Locals::default();
        locals.copy_from_slice(self.opstack.as_slice());
        self.opstack.clear();
        let entry = pc.next::<u64>()?;
        let ret = pc.position() as usize;
        let opstack = OperandStack::default();
        let frame = Frame::new(locals, opstack, entry, ret);
        Ok(FrameResult::Call(frame))
    }
}

pub struct Interpreter<'a> {
    pc: Program<'a>,
    frames: Vec<Frame>,
}

impl<'a> Interpreter<'a> {
    pub fn new(program: &'a [u8]) -> Result<Self> {
        let mut pc = Program::new(program);
        let entry = pc.next::<u64>()?;
        pc.set_position(entry);
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

            // TODO: better error reporting
            match current.run(&mut self.pc)? {
                FrameResult::Call(next) => {
                    self.pc.set_position(next.entry as u64);
                    self.frames.push(current);
                    self.frames.push(next);
                }
                FrameResult::Ret | FrameResult::RetW | FrameResult::RetD if is_entry => {
                    self.frames.push(current);
                    break;
                }
                FrameResult::Ret => {
                    self.pc.set_position(current.ret as u64);
                }
                FrameResult::RetW => {
                    self.pc.set_position(current.ret as u64);
                    self.frames[len - 1]
                        .opstack
                        .push::<i32>(current.opstack.pop());
                }
                FrameResult::RetD => {
                    self.pc.set_position(current.ret as u64);
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
