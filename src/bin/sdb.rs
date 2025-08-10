use std::env;
use std::fs::File;
use std::io::{stdin, stdout, Write};
use std::process;

use stack::debugger::Debugger;
use stack::output::Output;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// TODO: list breakpoints
enum Command {
    Run,
    Step,
    Continue,
    Stack,
    Peek,
    Break(u64),
    Variable(u64),
    Backtrace,
    Disassembly,
}

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

    // TODO: handle I/O better
    // TODO: handle continuing to a breakpoint better
    stdout.write_all(PROMPT)?;
    stdout.flush()?;
    while let Some(line) = stdin.next() {
        let line = line?;
        let Ok(command) = parse_command(&line) else {
            writeln!(stdout, "invalid command")?;
            stdout.write_all(PROMPT)?;
            stdout.flush()?;
            continue;
        };

        match command {
            Command::Run => 'run: {
                let position = match debugger.run() {
                    Ok(p) => p,
                    Err(e) => {
                        writeln!(stdout, "{e}")?;
                        break 'run;
                    }
                };

                debugger.fmt_line(&mut stdout, position)?;
            }
            Command::Step => 'step: {
                let position = match debugger.step() {
                    Ok(p) => p,
                    Err(e) => {
                        writeln!(stdout, "{e}")?;
                        break 'step;
                    }
                };

                debugger.fmt_line(&mut stdout, position)?;
            }
            Command::Continue => {
                if let Err(e) = debugger.r#continue() {
                    writeln!(stdout, "{e}")?;
                }
            }
            Command::Stack => writeln!(stdout, "{}", debugger.stack())?,
            Command::Peek => writeln!(stdout, "{:?}", debugger.peek::<i32>())?,
            Command::Break(position) => debugger.set_breakpoint(position)?,
            Command::Variable(variable) => {
                writeln!(stdout, "{}", debugger.variable::<i32>(variable))?;
            }
            Command::Backtrace => debugger.fmt_backtrace(&mut stdout)?,
            Command::Disassembly => write!(stdout, "{}", debugger.output())?,
        }

        stdout.write_all(PROMPT)?;
        stdout.flush()?;
    }

    Ok(())
}

fn parse_command(line: &str) -> Result<Command> {
    let mut parts = line.split_whitespace();

    let command = match parts.next().unwrap_or_default() {
        "r" | "run" => Command::Run,
        "s" | "step" | "" => Command::Step,
        "st" | "stack" => Command::Stack,
        "c" | "continue" => Command::Continue,
        "b" | "break" => {
            let Some(position) = parts.next() else {
                Err("could not parse argument")?
            };
            let position = position.parse::<u64>()?;
            Command::Break(position)
        }
        "v" | "var" => {
            let Some(variable) = parts.next() else {
                Err("could not parse argument")?
            };
            let variable = variable.parse::<u64>()?;
            Command::Variable(variable)
        }
        "p" | "peek" => Command::Peek,
        "bt" | "backtrace" => Command::Backtrace,
        "dis" | "disassembly" => Command::Disassembly,
        cmd => Err(format!("invalid command: {cmd}"))?,
    };

    Ok(command)
}
