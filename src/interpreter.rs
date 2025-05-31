use std::cmp::Ordering;
use std::io::{Cursor, Read};
use std::mem;

use crate::number::Number;
use crate::stack::Stack;

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

    Fail,
    Ret,
}

const LOCALS_SIZE: usize = mem::size_of::<i32>() * 128;
pub struct Interpreter<'a> {
    program: Cursor<&'a [u8]>,
    stack: Stack,
    locals: [u8; LOCALS_SIZE],
}

macro_rules! slot {
    ($ty:ty, $i:expr) => {{
        let from = $i * mem::size_of::<$ty>();
        let to = from + mem::size_of::<$ty>();
        from..to
    }};
}

impl<'a> Interpreter<'a> {
    pub fn new(stack: Stack, program: &'a [u8]) -> Self {
        let program = Cursor::new(program);
        let locals = [0; LOCALS_SIZE];
        Self {
            stack,
            program,
            locals,
        }
    }

    pub fn stack(&self) -> &Stack {
        &self.stack
    }

    fn next<const N: usize>(&mut self) -> Result<[u8; N]> {
        let mut buf = [0u8; N];
        let n = self.program.read(&mut buf)?;
        if n < N {
            assert_eq!(n, 0);
            Err(format!("read less than expected bytes: {n}"))?;
        }

        Ok(buf)
    }

    fn next_i32(&mut self) -> Result<i32> {
        const N: usize = size_of::<i32>();
        let buf = self.next::<N>()?;
        let val = i32::from_le_bytes(buf);
        Ok(val)
    }

    fn next_usize(&mut self) -> Result<usize> {
        const N: usize = size_of::<usize>();
        let buf = self.next::<N>()?;
        let val = usize::from_le_bytes(buf);
        Ok(val)
    }

    fn next_op(&mut self) -> Result<Bytecode> {
        const N: usize = size_of::<u8>();
        let buf = self.next::<N>()?;
        let op = u8::from_le_bytes(buf);
        assert!(op <= Bytecode::Ret as u8);
        let op = unsafe { std::mem::transmute::<_, Bytecode>(op) };
        Ok(op)
    }

    fn read_local<T: Number>(&self, i: usize) -> T {
        T::from_le_bytes(&self.locals[slot!(T, i)])
    }

    fn write_local<T: Number>(&mut self, i: usize, value: T) {
        self.locals[slot!(T, i)].copy_from_slice(value.to_le_bytes().as_ref());
    }

    pub fn run(&mut self) -> Result<()> {
        let start = self.next_i32()?;
        self.program.set_position(start.try_into().unwrap());

        loop {
            let op = self.next_op()?;

            match op {
                Bytecode::Push => {
                    let val = self.next_i32()?;
                    self.stack.push(val);
                }
                Bytecode::Pop => {
                    self.stack.pop();
                }
                Bytecode::Load => {
                    let i = self.next_usize()?;
                    let val = self.read_local::<i32>(i);
                    self.stack.push(val);
                }
                Bytecode::Store => {
                    let a = self.stack.pop();
                    let i = self.next_usize()?;
                    self.write_local::<i32>(i, a);
                }
                Bytecode::Add => self.stack.add(),
                Bytecode::Sub => self.stack.sub(),
                Bytecode::Mul => self.stack.mul(),
                Bytecode::Div => self.stack.div(),
                Bytecode::Cmp => {
                    let lhs = self.next_i32()?;
                    self.stack.cmp(lhs);
                }
                Bytecode::Jmp => {
                    let pos = self.next_usize()?;
                    self.program.set_position(pos.try_into().unwrap());
                }
                Bytecode::JmpEq => {
                    let pos = self.next_usize()?;
                    if self.stack.pop() == Ordering::Equal as i32 {
                        self.program.set_position(pos as u64);
                    }
                }
                Bytecode::JmpNe => {
                    let pos = self.next_usize()?;
                    if self.stack.pop() != Ordering::Equal as i32 {
                        self.program.set_position(pos.try_into().unwrap());
                    }
                }
                Bytecode::JmpLt => {
                    let pos = self.next_usize()?;
                    if self.stack.pop() == Ordering::Less as i32 {
                        self.program.set_position(pos as u64);
                    }
                }
                Bytecode::JmpGt => {
                    let pos = self.next_usize()?;
                    if self.stack.pop() == Ordering::Greater as i32 {
                        self.program.set_position(pos.try_into().unwrap());
                    }
                }
                Bytecode::Swap => self.stack.swap(),
                Bytecode::Dup => self.stack.dup(),
                Bytecode::Over => self.stack.over(),
                Bytecode::Rot => self.stack.rot(),
                Bytecode::Fail => Err("FAILED")?,
                Bytecode::Ret => break,
            }
        }

        Ok(())
    }
}
