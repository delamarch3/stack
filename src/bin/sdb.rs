use std::env;
use std::fs::File;
use std::io::{stdin, stdout, Write};
use std::process;

use stack::interpreter::Interpreter;
use stack::output::Output;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

enum State {
    Off,
    Running,
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
    let mut state = State::Off;
    let mut interpreter = Interpreter::new(&output)?;

    let mut text = String::new();
    let lines = output.fmt_text(&mut text)?;
    let text: Vec<&str> = text.split('\n').collect();

    let mut stdout = stdout();
    let mut stdin = stdin().lines();

    stdout.write_all(PROMPT)?;
    stdout.flush()?;
    while let Some(line) = stdin.next() {
        let line = line?;

        match line.as_str() {
            "r" | "run" => match state {
                State::Running => writeln!(stdout, "program currently running")?,
                State::Off => {
                    interpreter.reset();
                    let position = interpreter.position();
                    let line = lines[&position];
                    fmt_out(&mut stdout, &interpreter, &output, &text, line)?;
                    state = State::Running;
                }
            },
            "s" | "step" => match state {
                State::Running => {
                    let Some(position) = interpreter.step()? else {
                        writeln!(stdout, "program finished running")?;
                        continue;
                    };

                    let line = lines[&position];
                    fmt_out(&mut stdout, &interpreter, &output, &text, line)?;
                }
                State::Off => writeln!(stdout, "no program currently running")?,
            },
            "stack" => {
                writeln!(stdout, "{}", interpreter.frames().last().unwrap().opstack)?;
            }
            "c" | "continue" => match state {
                State::Running => {
                    interpreter.run()?;
                    state = State::Off;
                }
                State::Off => writeln!(stdout, "no program currently running")?,
            },
            // TODO: parse args
            "b" | "break" => todo!(),
            "v" | "var" => {
                writeln!(
                    stdout,
                    "{}",
                    interpreter.frames().last().unwrap().locals.read::<i32>(0)
                )?;
            }
            "p" | "peek" => {
                writeln!(
                    stdout,
                    "{:?}",
                    interpreter.frames().last().unwrap().opstack.peek::<i32>()
                )?;
            }
            "bt" | "backtrace" => {
                const TAB: usize = 2;
                let mut tab = 0;
                for (i, frame) in interpreter.frames().iter().enumerate() {
                    writeln!(
                        stdout,
                        "{:tab$}\x1b[94mFrame #{} `{}`\x1b[0m: Entry: {} Return: {}",
                        "",
                        i,
                        output.labels()[&frame.entry],
                        frame.entry,
                        frame.ret
                    )?;
                    tab += TAB;
                }
            }
            "dis" | "disassembly" => write!(stdout, "{output}")?,
            cmd => write!(stdout, "invalid command: {cmd}\n")?,
        }

        stdout.write_all(PROMPT)?;
        stdout.flush()?;
    }

    Ok(())
}

fn fmt_out(
    f: &mut impl std::io::Write,
    interpreter: &Interpreter,
    output: &Output,
    lines: &Vec<&str>,
    start: usize,
) -> Result<()> {
    const LOOK_FORWARD: usize = 4;
    const POINTER: &str = "->";
    const WIDTH: usize = 2;

    let mut end = start + LOOK_FORWARD;
    if end >= lines.len() {
        end = lines.len()
    }

    let frames = interpreter.frames();
    let entry = frames.last().unwrap().entry;

    writeln!(
        f,
        "\x1b[94mFrame #{} `{}`\x1b[0m",
        interpreter.frames().len() - 1,
        output.labels()[&entry]
    )?;
    for i in start..end {
        if i == start {
            writeln!(f, "\x1b[93m{POINTER:>WIDTH$}{}\x1b[0m", lines[i])?;
            continue;
        }

        writeln!(f, "{:WIDTH$}{}", "", lines[i])?;
    }

    Ok(())
}
