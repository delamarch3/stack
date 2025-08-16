use std::collections::{HashMap, HashSet};
use std::io::Write;

use crate::frame::Frame;
use crate::interpreter::Interpreter;
use crate::output::Output;
use crate::stack::OperandStack;
use crate::{Number, Result};

#[derive(Debug, Default)]
enum State {
    #[default]
    Off,
    Running,
}

pub struct Debugger {
    state: State,
    interpreter: Interpreter,
    output: Output,
    breakpoints: HashSet<u64>,
    /// The lines from the disassembly
    text: Vec<String>,
    /// Maps a position from the program to a line in [`Debugger::text`]
    lines: HashMap<u64, usize>,
}

impl Debugger {
    pub fn new(output: Output) -> Result<Self> {
        let interpreter = Interpreter::new(&output)?;
        let state = State::default();
        let breakpoints = HashSet::new();

        let mut text = String::new();
        let lines = output.fmt_text(&mut text)?;
        let text = text.lines().map(String::from).collect();

        Ok(Self {
            state,
            interpreter,
            output,
            breakpoints,
            text,
            lines,
        })
    }

    pub fn fmt_line(&self, w: &mut impl Write, position: u64) -> Result<()> {
        const LOOK_FORWARD: usize = 4;
        const POINTER: &str = "->";
        const WIDTH: usize = 2;

        let start = self.lines[&position];

        let mut end = start + LOOK_FORWARD;
        if end >= self.text.len() {
            end = self.text.len()
        }

        let frames = self.interpreter.frames();
        let entry = frames.last().unwrap().entry;

        writeln!(
            w,
            "\x1b[94mFrame #{} `{}`\x1b[0m",
            frames.len() - 1,
            self.output.labels()[&entry]
        )?;

        for i in start..end {
            if i == start {
                writeln!(w, "\x1b[93m{POINTER:>WIDTH$}{}\x1b[0m", self.text[i])?;
                continue;
            }

            writeln!(w, "{:WIDTH$}{}", "", self.text[i])?;
        }

        Ok(())
    }

    pub fn fmt_backtrace(&self, w: &mut impl Write) -> Result<()> {
        const TAB_SPACES: usize = 2;

        let mut tab = 0;
        for (i, frame) in self.interpreter.frames().iter().enumerate() {
            writeln!(
                w,
                "{:tab$}\x1b[94mFrame #{} `{}`\x1b[0m: Entry: {} Return: {}",
                "",
                i,
                self.output.labels()[&frame.entry],
                frame.entry,
                frame.ret
            )?;
            tab += TAB_SPACES;
        }

        Ok(())
    }

    pub fn fmt_breakpoints(&self, w: &mut impl Write) -> Result<()> {
        self.breakpoints
            .iter()
            .try_for_each(|bp| writeln!(w, "{}", self.text[self.lines[bp]]))?;

        Ok(())
    }

    pub fn run(&mut self) -> Result<u64> {
        if matches!(self.state, State::Running) {
            Err("program is currently running")?
        }

        self.interpreter.reset();
        let position = self.interpreter.position();
        self.state = State::Running;

        Ok(position)
    }

    pub fn step(&mut self) -> Result<u64> {
        if matches!(self.state, State::Off) {
            Err("no program currently running")?
        }

        let Some(position) = self.interpreter.step()? else {
            self.state = State::Off;
            Err("program finished running")?
        };

        Ok(position)
    }

    pub fn r#continue(&mut self) -> Result<u64> {
        if matches!(self.state, State::Off) {
            Err("no prgram currently running")?
        }

        let current_position = self.interpreter.position();
        let breakpoint = self.breakpoints.iter().find(|&&bp| bp >= current_position);
        let finished = match breakpoint {
            Some(&bp) => self.interpreter.run_until(bp)?,
            None => {
                self.interpreter.run()?;
                true
            }
        };
        if finished {
            self.state = State::Off;
        }

        Ok(self.interpreter.position())
    }

    pub fn set_breakpoint(&mut self, position: u64) -> Result<()> {
        match self.lines.get(&position) {
            Some(_) => self.breakpoints.insert(position),
            None => Err("invalid breakpoint, position must be at the start of an instruction")?,
        };

        Ok(())
    }

    pub fn delete_breakpoint(&mut self, position: u64) {
        self.lines.remove(&position);
    }

    pub fn output(&self) -> &Output {
        &self.output
    }

    pub fn stack(&self) -> &OperandStack {
        &self.current_frame().opstack
    }

    pub fn variable<N: Number>(&self, i: u64) -> N {
        self.current_frame().locals.read(i)
    }

    pub fn peek<N: Number>(&self) -> Option<N> {
        self.current_frame().opstack.peek()
    }

    fn current_frame(&self) -> &Frame {
        &self.interpreter.frames().last().unwrap()
    }
}
