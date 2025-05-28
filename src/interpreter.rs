use std::cmp::Ordering;
use std::io::{Cursor, Read};

use crate::stack::Stack;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum Bytecode {
    Push = 0,
    Pop,
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

pub struct Interpreter<'a> {
    program: Cursor<&'a [u8]>,
    stack: Stack,
}

impl<'a> Interpreter<'a> {
    pub fn new(stack: Stack, program: &'a [u8]) -> Self {
        let program = Cursor::new(program);
        Self { stack, program }
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

    fn next_value(&mut self) -> Result<i64> {
        const N: usize = size_of::<i64>();
        let buf = self.next::<N>()?;
        let val = i64::from_be_bytes(buf);
        Ok(val)
    }

    fn next_op(&mut self) -> Result<Bytecode> {
        const N: usize = size_of::<u8>();
        let buf = self.next::<N>()?;
        let op = u8::from_be_bytes(buf);
        assert!(op <= Bytecode::Ret as u8);
        let op = unsafe { std::mem::transmute::<_, Bytecode>(op) };
        Ok(op)
    }

    pub fn run(&mut self) -> Result<()> {
        let start = self.next_value()?;
        self.program.set_position(start.try_into().unwrap());

        loop {
            let op = self.next_op()?;

            match op {
                Bytecode::Push => {
                    let val = self.next_value()?;
                    self.stack.push(val);
                }
                Bytecode::Pop => {
                    self.stack.pop();
                }
                Bytecode::Add => self.stack.add(),
                Bytecode::Sub => self.stack.sub(),
                Bytecode::Mul => self.stack.mul(),
                Bytecode::Div => self.stack.div(),
                Bytecode::Cmp => {
                    let a = self.stack.pop();
                    let b = self.next_value()?;
                    let cmp = a.cmp(&b) as i64;
                    self.stack.push(cmp);
                }
                Bytecode::Jmp => {
                    let pos = self.next_value()?;
                    self.program.set_position(pos.try_into().unwrap());
                }
                Bytecode::JmpLt => {
                    let pos = self.next_value()?;
                    if self.stack.pop() == Ordering::Less as i64 {
                        self.program.set_position(pos.try_into().unwrap());
                    }
                }
                Bytecode::JmpEq => {
                    let pos = self.next_value()?;
                    if self.stack.pop() == Ordering::Equal as i64 {
                        self.program.set_position(pos.try_into().unwrap());
                    }
                }
                Bytecode::JmpGt => {
                    let pos = self.next_value()?;
                    if self.stack.pop() == Ordering::Greater as i64 {
                        self.program.set_position(pos.try_into().unwrap());
                    }
                }
                Bytecode::JmpNe => {
                    let pos = self.next_value()?;
                    if self.stack.pop() != Ordering::Equal as i64 {
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
