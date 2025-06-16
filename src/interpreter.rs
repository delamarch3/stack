use crate::frame::{Frame, FrameResult};
use crate::locals::Locals;
use crate::program::Program;
use crate::stack::OperandStack;
use crate::Result;

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
                    self.pc.set_position(next.entry);
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
