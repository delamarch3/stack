use std::collections::HashSet;
use std::sync::Arc;

use crate::frame::{Frame, FrameResult};
use crate::heap::Heap;
use crate::locals::Locals;
use crate::output::Output;
use crate::program::Program;
use crate::stack::OperandStack;
use crate::{Result, SharedWriter};

const MAIN_RETURN: u64 = 0;

pub enum ReturnFrom {
    Main,
    Other,
}

#[derive(Default)]
pub struct InterpreterOptions {
    argv: Option<Locals>,
    stdout: Option<SharedWriter>,
    stderr: Option<SharedWriter>,
}

impl InterpreterOptions {
    pub fn with_argv(mut self, argv: Locals) -> Self {
        self.argv.replace(argv);
        self
    }

    pub fn with_stdout(mut self, stdout: SharedWriter) -> Self {
        self.stdout.replace(stdout);
        self
    }

    pub fn with_stderr(mut self, stderr: SharedWriter) -> Self {
        self.stderr.replace(stderr);
        self
    }
}

pub struct Interpreter {
    entry: u64,
    pc: Program<Vec<u8>>,
    frames: Vec<Frame>,
    heap: Arc<Heap>,
    stdout: Option<SharedWriter>,
    stderr: Option<SharedWriter>,
}

impl Interpreter {
    pub fn new(output: &Output, options: InterpreterOptions) -> Result<Self> {
        let mut pc = Program::new(output);

        let entry = pc.next::<u64>()?;
        pc.set_position(entry);

        let heap = Arc::<Heap>::default();

        let main = Frame::new(
            options.argv.unwrap_or_default(),
            OperandStack::default(),
            Arc::clone(&heap),
            entry,
            MAIN_RETURN,
            options.stdout.as_ref().map(Arc::clone),
            options.stderr.as_ref().map(Arc::clone),
        );
        let frames = vec![main];

        Ok(Self {
            entry,
            pc,
            frames,
            heap,
            stdout: options.stdout,
            stderr: options.stderr,
        })
    }

    pub fn reset(&mut self) {
        self.pc.set_position(self.entry);
        self.frames.clear();

        let main = Frame::new(
            Locals::default(),
            OperandStack::default(),
            Arc::clone(&self.heap),
            self.entry,
            MAIN_RETURN,
            self.stdout.as_ref().map(Arc::clone),
            self.stderr.as_ref().map(Arc::clone),
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
            match self.handle_frame_result(fr, current)? {
                Some(ReturnFrom::Main) => break,
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
                Some(ReturnFrom::Main) => return Ok(None),
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
    ) -> Result<Option<ReturnFrom>> {
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
                // Make it appear as if the pc is still
                // pointing to the return instruction for
                // the debugger.
                self.pc.set_position(position);
                self.frames.push(current);
                Some(ReturnFrom::Main)
            }
            FrameResult::Ret(_) => {
                self.pc.set_position(current.ret);
                Some(ReturnFrom::Other)
            }
            FrameResult::RetW(_) => {
                self.pc.set_position(current.ret);
                self.frames[last].opstack.push::<i32>(current.opstack.pop());
                Some(ReturnFrom::Other)
            }
            FrameResult::RetD(_) => {
                self.pc.set_position(current.ret);
                self.frames[last].opstack.push::<i64>(current.opstack.pop());
                Some(ReturnFrom::Other)
            }
            FrameResult::Panic(_) => {
                // Push the frame back on so we can inspect it
                self.frames.push(current);
                Err("panic")?
            }
        };

        Ok(ret)
    }
}
