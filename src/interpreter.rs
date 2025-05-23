use std::cmp::Ordering;
use std::io::{self, Cursor, Read};

use crate::stack::Stack;

#[repr(i64)]
#[derive(Debug, PartialEq, Eq)]
pub enum Bytecode {
    Val = 0,
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

    fn next(&mut self) -> io::Result<Option<i64>> {
        let mut buf = [0u8; 8];
        let read = self.program.read(&mut buf)?;
        if read < 8 {
            assert_eq!(read, 0);
            return Ok(None);
        }

        let val = i64::from_be_bytes(buf);
        Ok(Some(val))
    }

    fn next_op(&mut self) -> io::Result<Option<Bytecode>> {
        let Some(op) = self.next()? else {
            return Ok(None);
        };

        assert!(op <= Bytecode::Ret as i64);
        let op = unsafe { std::mem::transmute::<_, Bytecode>(op) };
        Ok(Some(op))
    }

    pub fn run_program<R>(&mut self) -> io::Result<()> {
        loop {
            let Some(op) = self.next_op()? else { break };

            match op {
                Bytecode::Val => {
                    let val = self.next()?.expect("invalid program");
                    self.stack.push(val);
                }
                Bytecode::Add => self.stack.add(),
                Bytecode::Sub => self.stack.sub(),
                Bytecode::Mul => self.stack.mul(),
                Bytecode::Div => self.stack.div(),
                Bytecode::Cmp => {
                    let a = self.stack.pop();
                    let b = self.next()?.expect("invalid program");
                    let cmp = a.cmp(&b) as i64;
                    self.stack.push(a);
                    self.stack.push(cmp);
                }
                Bytecode::Jmp => {
                    let pos = self.next()?.expect("invalid program");
                    self.program.set_position(pos.try_into().unwrap());
                }
                Bytecode::JmpLt => {
                    let pos = self.next()?.expect("invalid program");
                    if self.stack.pop() == Ordering::Less as i64 {
                        self.program.set_position(pos.try_into().unwrap());
                    }
                }
                Bytecode::JmpEq => {
                    let pos = self.next()?.expect("invalid program");
                    if self.stack.pop() == Ordering::Equal as i64 {
                        self.program.set_position(pos.try_into().unwrap());
                    }
                }
                Bytecode::JmpGt => {
                    let pos = self.next()?.expect("invalid program");
                    if self.stack.pop() == Ordering::Greater as i64 {
                        self.program.set_position(pos.try_into().unwrap());
                    }
                }
                Bytecode::Ret => break,
            }
        }

        Ok(())
    }
}
