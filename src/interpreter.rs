use std::collections::HashSet;

use crate::frame::{Frame, FrameResult};
use crate::locals::Locals;
use crate::output::Output;
use crate::program::Program;
use crate::stack::OperandStack;
use crate::Result;

const MAIN_RETURN: u64 = 0;

pub enum Return {
    Main,
    Other,
}

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

    /// Returns the position of the return instruction
    pub fn run(&mut self) -> Result<()> {
        while let Some(mut current) = self.frames.pop() {
            let fr = current.run(&mut self.pc)?;
            match self.handle_frame_result(fr, current)? {
                Some(Return::Main) => break,
                _ => {}
            }
        }

        Ok(())
    }

    /// Returns true if returning from the main routine
    pub fn run_until(&mut self, breakpoints: &HashSet<u64>) -> Result<bool> {
        loop {
            let Some(position) = self.step()? else {
                return Ok(true);
            };

            if breakpoints.contains(&position) {
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
            match self.handle_frame_result(fr, current)? {
                Some(Return::Main) => return Ok(None),
                _ => {}
            }
        } else {
            self.frames.push(current);
        }

        Ok(Some(self.pc.position()))
    }

    fn handle_frame_result(
        &mut self,
        fr: FrameResult,
        mut current: Frame,
    ) -> Result<Option<Return>> {
        let last = self.frames.len().saturating_sub(1);
        let main = self.entry == current.entry;

        let ret = match fr {
            FrameResult::Call(next) => {
                self.pc.set_position(next.entry);
                self.frames.push(current);
                self.frames.push(next);
                None
            }
            FrameResult::Ret(position)
            | FrameResult::RetW(position)
            | FrameResult::RetD(position)
                if main =>
            {
                // Make it appear as if the pc is pointing to the return instruction
                self.pc.set_position(position);
                self.frames.push(current);
                Some(Return::Main)
            }
            FrameResult::Ret(_) => {
                self.pc.set_position(current.ret);
                Some(Return::Other)
            }
            FrameResult::RetW(_) => {
                self.pc.set_position(current.ret);
                self.frames[last].opstack.push::<i32>(current.opstack.pop());
                Some(Return::Other)
            }
            FrameResult::RetD(_) => {
                self.pc.set_position(current.ret);
                self.frames[last].opstack.push::<i64>(current.opstack.pop());
                Some(Return::Other)
            }
            FrameResult::Panic(_) => Err("panic")?,
        };

        Ok(ret)
    }
}
