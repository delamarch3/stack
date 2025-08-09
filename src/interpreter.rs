use crate::frame::{Frame, FrameResult};
use crate::locals::Locals;
use crate::output::Output;
use crate::program::Program;
use crate::stack::OperandStack;
use crate::Result;

const MAIN_RETURN: u64 = 0;

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

        let main = Frame::new(
            Locals::default(),
            OperandStack::default(),
            entry,
            MAIN_RETURN,
        );
        let frames = vec![main];

        Ok(Self { entry, pc, frames })
    }

    pub fn reset(&mut self) {
        self.pc.set_position(self.entry);
        self.frames.clear();

        let main = Frame::new(
            Locals::default(),
            OperandStack::default(),
            self.entry,
            MAIN_RETURN,
        );

        self.frames.push(main)
    }

    pub fn position(&self) -> u64 {
        self.pc.position()
    }

    pub fn frames(&self) -> &Vec<Frame> {
        &self.frames
    }

    pub fn run(&mut self) -> Result<()> {
        while let Some(mut current) = self.frames.pop() {
            let fr = current.run(&mut self.pc)?;
            if self.handle_frame_result(fr, current)? {
                break;
            }
        }

        Ok(())
    }

    /// Returns true if returning from the main routine
    pub fn run_until(&mut self, until: u64) -> Result<bool> {
        loop {
            let Some(position) = self.step()? else {
                return Ok(true);
            };

            if position >= until {
                break;
            }
        }

        Ok(false)
    }

    /// Results None if returning from the main routine
    pub fn step(&mut self) -> Result<Option<u64>> {
        let Some(mut current) = self.frames.pop() else {
            unreachable!()
        };

        if let Some(fr) = current.step(&mut self.pc)? {
            if self.handle_frame_result(fr, current)? {
                return Ok(None);
            }
        } else {
            self.frames.push(current);
        }

        Ok(Some(self.pc.position()))
    }

    /// Returns true if returning from the main routine
    fn handle_frame_result(&mut self, fr: FrameResult, mut current: Frame) -> Result<bool> {
        let last = self.frames.len().saturating_sub(1);
        let main = self.entry == current.entry;

        let return_main = match fr {
            FrameResult::Call(next) => {
                self.pc.set_position(next.entry);
                self.frames.push(current);
                self.frames.push(next);
                false
            }
            FrameResult::Ret | FrameResult::RetW | FrameResult::RetD if main => {
                self.frames.push(current);
                true
            }
            FrameResult::Ret => {
                self.pc.set_position(current.ret);
                false
            }
            FrameResult::RetW => {
                self.pc.set_position(current.ret);
                self.frames[last].opstack.push::<i32>(current.opstack.pop());
                false
            }
            FrameResult::RetD => {
                self.pc.set_position(current.ret);
                self.frames[last].opstack.push::<i64>(current.opstack.pop());
                false
            }
            FrameResult::Panic => Err("panic")?,
        };

        Ok(return_main)
    }
}
