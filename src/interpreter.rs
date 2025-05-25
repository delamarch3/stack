use std::cmp::Ordering;
use std::io::{Cursor, Read};

use crate::stack::Stack;

#[repr(i64)]
#[derive(Debug, PartialEq, Eq)]
pub enum Bytecode {
    Push = 0,
    Add,
    Sub,
    Mul,
    Div,
    Cmp,
    Jmp,
    JmpLt,
    JmpEq,
    JmpGt,

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

    fn next(&mut self) -> Option<i64> {
        let mut buf = [0u8; 8];
        let n = self.program.read(&mut buf).ok()?;
        if n < 8 {
            assert_eq!(n, 0);
            return None;
        }

        let val = i64::from_be_bytes(buf);
        Some(val)
    }

    fn next_op(&mut self) -> Option<Bytecode> {
        let op = self.next()?;
        assert!(op <= Bytecode::Ret as i64);
        let op = unsafe { std::mem::transmute::<_, Bytecode>(op) };
        Some(op)
    }

    pub fn run(mut self) -> Option<Stack> {
        let start = self.next()?;
        self.program.set_position(start as u64);

        loop {
            let op = self.next_op()?;

            match op {
                Bytecode::Push => {
                    let val = self.next()?;
                    self.stack.push(val);
                }
                Bytecode::Add => self.stack.add(),
                Bytecode::Sub => self.stack.sub(),
                Bytecode::Mul => self.stack.mul(),
                Bytecode::Div => self.stack.div(),
                Bytecode::Cmp => {
                    let a = self.stack.pop();
                    let b = self.next()?;
                    let cmp = a.cmp(&b) as i64;
                    self.stack.push(a);
                    self.stack.push(cmp);
                }
                Bytecode::Jmp => {
                    let pos = self.next()?;
                    self.program.set_position(pos.try_into().unwrap());
                }
                Bytecode::JmpLt => {
                    let pos = self.next()?;
                    if self.stack.pop() == Ordering::Less as i64 {
                        self.program.set_position(pos.try_into().unwrap());
                    }
                }
                Bytecode::JmpEq => {
                    let pos = self.next()?;
                    if self.stack.pop() == Ordering::Equal as i64 {
                        self.program.set_position(pos.try_into().unwrap());
                    }
                }
                Bytecode::JmpGt => {
                    let pos = self.next()?;
                    if self.stack.pop() == Ordering::Greater as i64 {
                        self.program.set_position(pos.try_into().unwrap());
                    }
                }
                Bytecode::Ret => break,
            }
        }

        Some(self.stack)
    }
}
