use std::env;
use std::fs::File;
use std::io::{stdin, stdout, Write};
use std::process;

use stack::interpreter::Interpreter;
use stack::output::Output;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let mut args = env::args();
    let program = args.next().unwrap();
    let Some(path) = args.next() else {
        eprintln!("usage: {} path/to/file", program);
        process::exit(1);
    };

    let file = File::open(path)?;
    let output = Output::deserialise(file)?;
    let mut interpreter = None;

    let mut text = String::new();
    let lines = output.fmt_text(&mut text)?;
    let text: Vec<&str> = text.split('\n').collect();

    let mut stdout = stdout();
    let mut stdin = stdin().lines();
    while let Some(line) = stdin.next() {
        let line = line?;

        match line.as_str() {
            "r" | "run" => match interpreter {
                Some(_) => writeln!(stdout, "program currently running...")?,
                None => {
                    let int = Interpreter::new(&output)?;
                    let position = int.position();
                    let line = lines[&position];
                    fmt_line(&mut stdout, &text, line)?;
                    interpreter = Some(int)
                }
            },
            "s" | "step" => match &mut interpreter {
                Some(int) => {
                    let Some(position) = int.step()? else {
                        writeln!(stdout, "program finished running")?;
                        interpreter = None;
                        continue;
                    };

                    let line = lines[&position];
                    fmt_line(&mut stdout, &text, line)?;
                }
                None => writeln!(stdout, "no program currently running")?,
            },
            "stack" => match interpreter {
                Some(ref i) => {
                    writeln!(stdout, "{}", i.opstack().unwrap())?;
                }
                None => writeln!(stdout, "no program currently running")?,
            },
            "v" | "var" => todo!(),
            "p" | "peek" => todo!(),
            "bt" | "backtrace" => todo!(),
            "disasm" => write!(stdout, "{output}")?,
            cmd => write!(stdout, "invalid command: {cmd}\n")?,
        }
    }

    Ok(())
}

fn fmt_line(f: &mut impl std::io::Write, lines: &Vec<&str>, line: usize) -> Result<()> {
    const PAD_LINES: usize = 3;
    const POINTER: &str = ">";
    const WIDTH: usize = 4;

    let start = line.saturating_sub(PAD_LINES);
    let mut end = line + 1 + PAD_LINES;
    if end >= lines.len() {
        end = lines.len()
    }

    for i in start..end {
        if i == line {
            writeln!(f, "\x1b[93m{POINTER:>WIDTH$}{}\x1b[0m", lines[i])?;
            continue;
        }

        writeln!(f, "{:WIDTH$}{}", "", lines[i])?;
    }

    Ok(())
}
