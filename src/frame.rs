use std::cmp::Ordering;

use crate::locals::Locals;
use crate::program::{Bytecode, Program};
use crate::stack::OperandStack;
use crate::{Number, Result};

pub(crate) struct Frame {
    pub opstack: OperandStack,
    pub locals: Locals,
    pub entry: u64,
    pub ret: u64,
}

pub(crate) enum FrameResult {
    Call(Frame),
    Ret,
    RetW,
    RetD,
    Fail,
}

impl Frame {
    pub fn new(locals: Locals, opstack: OperandStack, entry: u64, ret: u64) -> Self {
        Self {
            opstack,
            locals,
            entry,
            ret,
        }
    }

    pub fn run(&mut self, pc: &mut Program<Vec<u8>>) -> Result<FrameResult> {
        loop {
            if let Some(fr) = self.step(pc)? {
                return Ok(fr);
            }
        }
    }

    pub fn step(&mut self, pc: &mut Program<Vec<u8>>) -> Result<Option<FrameResult>> {
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

    fn push<T: Number>(&mut self, pc: &mut Program<Vec<u8>>) -> Result<()> {
        let val = pc.next::<T>()?;
        self.opstack.push(val);
        Ok(())
    }

    fn load<T: Number>(&mut self, pc: &mut Program<Vec<u8>>) -> Result<()> {
        let i = pc.next::<u64>()?;
        let val = self.locals.read::<T>(i);
        self.opstack.push(val);
        Ok(())
    }

    fn store<T: Number>(&mut self, pc: &mut Program<Vec<u8>>) -> Result<()> {
        let i = pc.next::<u64>()?;
        let val = self.opstack.pop();
        self.locals.write::<T>(i, val);
        Ok(())
    }

    fn get<T: Number>(&mut self, pc: &mut Program<Vec<u8>>) {
        let offset = self.opstack.pop::<u64>();
        let ptr = self.opstack.pop::<u64>();
        let value = pc.get::<T>((ptr + offset) as usize);
        self.opstack.push(value);
    }

    fn jmp(&mut self, pc: &mut Program<Vec<u8>>, conditions: &[Ordering]) -> Result<()> {
        let pos = pc.next::<u64>()?;

        let jmp = match conditions {
            [] => true,
            cs => {
                let have = self.opstack.pop::<i32>();
                cs.iter().any(|&want| want as i32 == have)
            }
        };

        if jmp {
            pc.set_position(pos);
        }

        Ok(())
    }

    fn call(&mut self, pc: &mut Program<Vec<u8>>) -> Result<FrameResult> {
        let mut locals = Locals::default();
        locals.copy_from_slice(self.opstack.as_slice());
        self.opstack.clear();
        let entry = pc.next::<u64>()?;
        let ret = pc.position();
        let opstack = OperandStack::default();
        let frame = Frame::new(locals, opstack, entry, ret);
        Ok(FrameResult::Call(frame))
    }
}
