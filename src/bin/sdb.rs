use std::env;
use std::fs::File;
use std::io::{stdin, stdout, Write};
use std::process;

use stack::debugger::Debugger;
use stack::output::Output;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    const PROMPT: &[u8; 15] = b"\x1b[90m(sdb)\x1b[0m ";

    let mut args = env::args();
    let program = args.next().unwrap();
    let Some(path) = args.next() else {
        eprintln!("usage: {} path/to/file", program);
        process::exit(1);
    };

    let file = File::open(path)?;
    let output = Output::deserialise(file)?;
    let mut debugger = Debugger::new(output)?;

    let mut stdout = stdout();
    let mut stdin = stdin().lines();

    stdout.write_all(PROMPT)?;
    stdout.flush()?;
    while let Some(line) = stdin.next() {
        let line = line?;

        match line.as_str() {
            "r" | "run" => 'run: {
                let position = match debugger.run() {
                    Ok(p) => p,
                    Err(e) => {
                        writeln!(stdout, "{e}")?;
                        break 'run;
                    }
                };

                debugger.fmt_line(&mut stdout, position)?;
            }
            "s" | "step" | "" => 'step: {
                let position = match debugger.step() {
                    Ok(p) => p,
                    Err(e) => {
                        writeln!(stdout, "{e}")?;
                        break 'step;
                    }
                };

                debugger.fmt_line(&mut stdout, position)?;
            }
            "stack" => {
                writeln!(stdout, "{}", debugger.stack())?;
            }
            "c" | "continue" => {
                if let Err(e) = debugger.r#continue() {
                    writeln!(stdout, "{e}")?;
                }
            }
            // TODO: parse args
            "b" | "break" => todo!(),
            "v" | "var" => {
                writeln!(stdout, "{}", debugger.variable::<i32>(0))?;
            }
            "p" | "peek" => {
                writeln!(stdout, "{:?}", debugger.peek::<i32>())?;
            }
            "bt" | "backtrace" => {
                debugger.fmt_backtrace(&mut stdout)?;
            }
            "dis" | "disassembly" => write!(stdout, "{}", debugger.output())?,
            cmd => writeln!(stdout, "invalid command: {cmd}")?,
        }

        stdout.write_all(PROMPT)?;
        stdout.flush()?;
    }

    Ok(())
}
