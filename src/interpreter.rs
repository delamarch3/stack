use crate::frame::{Frame, FrameResult};
use crate::locals::Locals;
use crate::output::Output;
use crate::program::Program;
use crate::stack::OperandStack;
use crate::Result;

pub struct Interpreter {
    entry: u64,
    pc: Program<Vec<u8>>,
    frames: Vec<Frame>,
}

impl Interpreter {
    pub fn new(output: &Output) -> Result<Self> {
        let mut pc = Program::new(output.into());

        let entry = pc.next::<u64>()?;
        pc.set_position(entry);

        let opstack = OperandStack::default();
        let locals = Locals::default();

        let ret = 0;
        let main = Frame::new(locals, opstack, entry, ret);
        let frames = vec![main];

        Ok(Self { entry, pc, frames })
    }

    pub fn print_opstack(&self) {
        println!("{}", self.frames.last().unwrap().opstack)
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
                    self.pc.set_position(current.ret);
                }
                FrameResult::RetW => {
                    self.pc.set_position(current.ret);
                    self.frames[len - 1]
                        .opstack
                        .push::<i32>(current.opstack.pop());
                }
                FrameResult::RetD => {
                    self.pc.set_position(current.ret);
                    self.frames[len - 1]
                        .opstack
                        .push::<i64>(current.opstack.pop());
                }
                FrameResult::Fail => Err("FAILED")?,
            }
        }

        Ok(())
    }

    pub fn step(&mut self) -> Result<u64> {
        let Some(mut current) = self.frames.pop() else {
            unreachable!()
        };

        let len = self.frames.len();
        let is_entry = len == 0;

        if let Some(fr) = current.step(&mut self.pc)? {
            match fr {
                FrameResult::Call(next) => {
                    self.pc.set_position(next.entry);
                    self.frames.push(current);
                    self.frames.push(next);
                }
                FrameResult::Ret | FrameResult::RetW | FrameResult::RetD if is_entry => {
                    self.frames.push(current);
                }
                FrameResult::Ret => {
                    self.pc.set_position(current.ret);
                }
                FrameResult::RetW => {
                    self.pc.set_position(current.ret);
                    self.frames[len - 1]
                        .opstack
                        .push::<i32>(current.opstack.pop());
                }
                FrameResult::RetD => {
                    self.pc.set_position(current.ret);
                    self.frames[len - 1]
                        .opstack
                        .push::<i64>(current.opstack.pop());
                }
                FrameResult::Fail => Err("FAILED")?,
            }
        } else {
            self.frames.push(current);
        }

        Ok(self.pc.position())
    }
}
