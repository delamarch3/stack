use std::cmp::Ordering;
use std::fs::File;
use std::io::{Read, Write};
use std::mem;
use std::os::fd::FromRawFd;
use std::sync::Arc;

use crate::heap::Heap;
use crate::locals::Locals;
use crate::program::{Bytecode, Program};
use crate::stack::OperandStack;
use crate::{Number, Result};

pub enum FrameResult {
    Call(Frame),

    // The following hold the position of their instruction
    Ret(u64),
    RetW(u64),
    RetD(u64),
    Panic(u64),
}

pub struct Frame {
    pub opstack: OperandStack,
    pub locals: Locals,
    heap: Arc<Heap>,
    /// The position of the first instruction of the frame
    pub entry: u64,
    /// The position of the first instruction after the call
    pub ret: u64,
}

impl Frame {
    pub fn new(
        locals: Locals,
        opstack: OperandStack,
        heap: Arc<Heap>,
        entry: u64,
        ret: u64,
    ) -> Self {
        Self {
            opstack,
            locals,
            heap,
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
        let position = pc.position();

        match pc.next_op()? {
            Bytecode::ALoad => self.aload::<i32>()?,
            Bytecode::ALoadB => self.aload::<i8>()?,
            Bytecode::ALoadD => self.aload::<i64>()?,
            Bytecode::AStore => self.astore::<i32>()?,
            Bytecode::AStoreB => self.astore::<i8>()?,
            Bytecode::AStoreD => self.astore::<i64>()?,
            Bytecode::Add => self.opstack.add::<i32>(),
            Bytecode::AddB => self.opstack.add::<i8>(),
            Bytecode::AddD => self.opstack.add::<i64>(),
            Bytecode::Alloc => self.alloc()?,
            Bytecode::Cmp => self.opstack.cmp::<i32>(),
            Bytecode::CmpD => self.opstack.cmp::<i64>(),
            Bytecode::DataPtr => self.dataptr(pc)?,
            Bytecode::Div => self.opstack.div::<i32>(),
            Bytecode::DivD => self.opstack.div::<i64>(),
            Bytecode::Dup => self.opstack.dup::<i32>(),
            Bytecode::DupD => self.opstack.dup::<i64>(),
            Bytecode::Free => self.free()?,
            Bytecode::Get => self.get::<i32>(pc),
            Bytecode::GetB => self.get::<i8>(pc),
            Bytecode::GetD => self.get::<i64>(pc),
            Bytecode::Jmp => self.jmp(pc, &[])?,
            Bytecode::JmpEq => self.jmp(pc, &[Ordering::Equal])?,
            Bytecode::JmpGe => self.jmp(pc, &[Ordering::Greater, Ordering::Equal])?,
            Bytecode::JmpGt => self.jmp(pc, &[Ordering::Greater])?,
            Bytecode::JmpLe => self.jmp(pc, &[Ordering::Less, Ordering::Equal])?,
            Bytecode::JmpLt => self.jmp(pc, &[Ordering::Less])?,
            Bytecode::JmpNe => self.jmp(pc, &[Ordering::Greater, Ordering::Less])?,
            Bytecode::Load => self.load::<i32>(pc)?,
            Bytecode::LoadB => self.load::<i8>(pc)?,
            Bytecode::LoadD => self.load::<i64>(pc)?,
            Bytecode::Mul => self.opstack.mul::<i32>(),
            Bytecode::MulD => self.opstack.mul::<i64>(),
            Bytecode::Pop => self.opstack.drop::<i32>(),
            Bytecode::PopB => self.opstack.drop::<i8>(),
            Bytecode::PopD => self.opstack.drop::<i64>(),
            Bytecode::Push => self.push::<i32>(pc)?,
            Bytecode::PushB => self.push::<i8>(pc)?,
            Bytecode::PushD => self.push::<i64>(pc)?,
            Bytecode::Store => self.store::<i32>(pc)?,
            Bytecode::StoreB => self.store::<i8>(pc)?,
            Bytecode::StoreD => self.store::<i64>(pc)?,
            Bytecode::Sub => self.opstack.sub::<i32>(),
            Bytecode::SubB => self.opstack.sub::<i8>(),
            Bytecode::SubD => self.opstack.sub::<i64>(),
            Bytecode::System => self.system()?,

            Bytecode::Call => return self.call(pc).map(Some),
            Bytecode::Panic => return Ok(Some(FrameResult::Panic(position))),
            Bytecode::Ret => return Ok(Some(FrameResult::Ret(position))),
            Bytecode::RetW => return Ok(Some(FrameResult::RetW(position))),
            Bytecode::RetD => return Ok(Some(FrameResult::RetD(position))),
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

        let jmp = conditions.is_empty() || {
            let have = self.opstack.pop::<i32>();
            conditions.iter().any(|&want| want as i32 == have)
        };

        if jmp {
            pc.set_position(pos);
        }

        Ok(())
    }

    fn alloc(&mut self) -> Result<()> {
        let size = self.opstack.pop::<u64>();
        let ptr = self.heap.alloc(size as usize);
        self.opstack.push(ptr as u64);

        Ok(())
    }

    fn free(&mut self) -> Result<()> {
        let ptr = self.opstack.pop::<u64>();
        self.heap.free(ptr as *const u8);

        Ok(())
    }

    fn dataptr(&mut self, pc: &mut Program<Vec<u8>>) -> Result<()> {
        let offset = pc.next::<u64>()?;
        let ptr = pc.getptr(offset as usize);
        self.opstack.push(ptr as u64);

        Ok(())
    }

    fn astore<T: Number>(&mut self) -> Result<()> {
        let data = self.opstack.pop::<T>();
        let offset = self.opstack.pop::<u64>();
        let ptr = self.opstack.pop::<u64>();
        let src = data.to_le_bytes();

        if !self
            .heap
            .write(ptr as *const u8, offset as usize, src.as_ref())
        {
            Err("{id}: no write")?;
        }

        Ok(())
    }

    fn aload<T: Number>(&mut self) -> Result<()> {
        let offset = self.opstack.pop::<u64>();
        let ptr = self.opstack.pop::<u64>();
        let mut dst = T::default().to_le_bytes();

        if !self
            .heap
            .read(ptr as *const u8, offset as usize, dst.as_mut())
        {
            Err("{id}: no read")?;
        }

        self.opstack.push(T::from_le_bytes(dst.as_ref()));

        Ok(())
    }

    fn system(&mut self) -> Result<()> {
        // Using the same system call numbers as https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/syscalls.master
        const EXIT: i32 = 1;
        const READ: i32 = 3;
        const WRITE: i32 = 4;
        const OPEN: i32 = 5;
        const CLOSE: i32 = 6;
        const FSYNC: i32 = 95;

        let call = self.opstack.pop::<i32>();

        match call {
            EXIT => {
                let code = self.opstack.pop::<i32>();
                std::process::exit(code)
            }
            READ => {
                let size = self.opstack.pop::<u64>() as usize;
                let ptr = self.opstack.pop::<u64>() as *mut u8;
                let fd = self.opstack.pop::<i32>();

                if ptr.is_null() {
                    Err("invalid ptr")?
                }

                let mut f = unsafe { File::from_raw_fd(fd) };
                let s = unsafe { std::slice::from_raw_parts_mut(ptr, size) };

                let r = match f.read(s) {
                    Ok(n) => n as i32,
                    Err(_) => -1,
                };

                self.opstack.push(r);

                // Avoid closing the file descriptor
                mem::forget(f);
            }
            WRITE => {
                let size = self.opstack.pop::<u64>() as usize;
                let ptr = self.opstack.pop::<u64>() as *const u8;
                let fd = self.opstack.pop::<i32>();

                if ptr.is_null() {
                    Err("invalid ptr")?
                }

                let mut f = unsafe { File::from_raw_fd(fd) };
                let s = unsafe { std::slice::from_raw_parts(ptr, size) };

                let r = match f.write(s) {
                    Ok(n) => n as i32,
                    Err(_) => -1,
                };

                self.opstack.push(r);

                // Avoid closing the file descriptor
                mem::forget(f);
            }
            OPEN => todo!(),
            CLOSE => {
                let fd = self.opstack.pop::<i32>();

                // Dropping the file will close it
                unsafe { File::from_raw_fd(fd) };
            }
            FSYNC => {
                let fd = self.opstack.pop::<i32>();

                let f = unsafe { File::from_raw_fd(fd) };

                let r = if let Err(_) = f.sync_all() { -1 } else { 0 };

                self.opstack.push::<i32>(r);
            }
            _ => Err(format!("invalid system call: {call}"))?,
        };

        Ok(())
    }

    fn call(&mut self, pc: &mut Program<Vec<u8>>) -> Result<FrameResult> {
        let mut locals = Locals::default();
        locals.copy_from_slice(self.opstack.as_slice());
        self.opstack.clear(); // TODO: would be nicer to avoid clearing the opstack

        let entry = pc.next::<u64>()?;
        let ret = pc.position();
        let opstack = OperandStack::default();
        let heap = Arc::clone(&self.heap);
        let frame = Frame::new(locals, opstack, heap, entry, ret);

        Ok(FrameResult::Call(frame))
    }
}
