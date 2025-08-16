use std::env;
use std::fs::File;
use std::io::{stdin, stdout, Stdout, Write};
use std::process;

use stack::debugger::Debugger;
use stack::output::Output;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

enum Command {
    Run,
    Step,
    Continue,
    Stack,
    Peek,
    BreakPosition(u64),
    BreakLabel(String),
    Delete(u64),
    List,
    Variable(u64),
    Backtrace,
    Disassembly,
}

fn main() -> Result<()> {
    const PROMPT: &str = "\x1b[90m(sdb)\x1b[0m ";

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

    stdout.write_fmt(format_args!("{PROMPT}"))?;
    stdout.flush()?;
    while let Some(line) = stdin.next() {
        let line = line?;

        if let Err(e) = parse_evaluate(&mut stdout, &mut debugger, line) {
            writeln!(stdout, "error: {e}")?;
        }

        stdout.write_fmt(format_args!("{PROMPT}"))?;
        stdout.flush()?;
    }

    Ok(())
}

fn parse_evaluate(stdout: &mut Stdout, debugger: &mut Debugger, line: String) -> Result<()> {
    let command = parse_command(&line)?;

    match command {
        Command::Run => {
            let position = debugger.run()?;
            debugger.fmt_line(stdout, position)?;
        }
        Command::Step => {
            let position = debugger.step()?;
            debugger.fmt_line(stdout, position)?;
        }
        Command::Continue => {
            let position = debugger.r#continue()?;
            debugger.fmt_line(stdout, position)?;
        }
        Command::Stack => writeln!(stdout, "{}", debugger.stack())?,
        Command::Peek => writeln!(stdout, "{:?}", debugger.peek::<i32>())?,
        Command::BreakPosition(position) => debugger.set_breakpoint(position)?,
        Command::BreakLabel(label) => debugger.set_label_breakpoint(&label)?,
        Command::Delete(position) => debugger.delete_breakpoint(position),
        Command::List => debugger.fmt_breakpoints(stdout)?,
        Command::Variable(variable) => {
            writeln!(stdout, "{}", debugger.variable::<i32>(variable))?;
        }
        Command::Backtrace => debugger.fmt_backtrace(stdout)?,
        Command::Disassembly => write!(stdout, "{}", debugger.output())?,
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
            let Some(arg) = parts.next() else {
                Err("could not parse argument")?
            };

            match arg.parse::<u64>() {
                Ok(position) => Command::BreakPosition(position),
                Err(_) => Command::BreakLabel(arg.into()),
            }
        }
        "d" => {
            let Some(position) = parts.next() else {
                Err("could not parse argument")?
            };
            let position = position.parse::<u64>()?;
            Command::Delete(position)
        }
        "ls" => Command::List,
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
