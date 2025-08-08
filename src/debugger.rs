use std::io::{BufRead, Lines, Write};

use crate::interpreter::Interpreter;
use crate::output::Output;
use crate::Result;

pub struct Debugger<R, W> {
    interpreter: Option<Interpreter>,
    output: Output,
    r: Lines<R>,
    w: W,
}

impl<R, W> Debugger<R, W>
where
    R: BufRead,
    W: Write,
{
    pub fn new(output: Output, r: Lines<R>, w: W) -> Self {
        let interpreter = None;

        Self {
            interpreter,
            output,
            r,
            w,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let None = self.interpreter else {
            return writeln!(self.w, "program currently running").map_err(Into::into);
        };

        self.interpreter = Some(Interpreter::new(&self.output)?);

        Ok(())
    }

    pub fn step(&mut self) -> Result<()> {
        let Some(ref mut interpreter) = self.interpreter else {
            return writeln!(self.w, "no program currently running").map_err(Into::into);
        };

        // let position = interpreter.step()?;
        // writeln!(self.w, "New position: {position}")?;

        Ok(())
    }
}
