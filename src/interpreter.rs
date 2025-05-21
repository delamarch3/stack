use std::io::{self, Cursor, Read};

use crate::stack::Stack;

// |........|........|
// ^ op     ^ val (if op == 0)
#[repr(i64)]
#[derive(Debug, PartialEq, Eq)]
pub enum Bytecode {
    Val = 0,
    Add,
    Sub,
    Mul,
    Div,
}
// TODO: implement loops with jmp
// TODO: implement cmp

pub struct Interpreter {
    stack: Stack,
}

impl Interpreter {
    pub fn new(stack: Stack) -> Self {
        Self { stack }
    }

    // TODO: full program will be in memory, no need to Read
    pub fn run_program<R>(&mut self, program: R) -> io::Result<()>
    where
        R: Read + AsRef<[u8]>,
    {
        let mut buf = [0u8; 8];
        let mut cursor = Cursor::new(program);

        loop {
            let read = cursor.read(&mut buf)?;
            if read < 8 {
                assert_eq!(read, 0);
                break;
            }
            let op = i64::from_be_bytes(buf);

            match op {
                op if op == Bytecode::Val as i64 => {
                    let read = cursor.read(&mut buf)?;
                    if read < 8 {
                        assert_eq!(read, 0);
                        break;
                    }
                    let val = i64::from_be_bytes(buf);

                    self.stack.push(val);
                }
                op if op == Bytecode::Add as i64 => self.stack.add(),
                op if op == Bytecode::Sub as i64 => self.stack.sub(),
                op if op == Bytecode::Mul as i64 => self.stack.mul(),
                op if op == Bytecode::Div as i64 => self.stack.div(),
                other => panic!("unknown op: {other}"),
            }
        }

        Ok(())
    }
}
