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
    let line_map = output.fmt_text(&mut text)?;
    let text: Vec<&str> = text.split('\n').collect();

    let mut stdout = stdout();
    let mut stdin = stdin().lines();
    while let Some(line) = stdin.next() {
        let line = line?;

        match line.as_str() {
            "r" | "run" => match interpreter {
                Some(_) => writeln!(stdout, "program currently running...")?,
                None => {
                    writeln!(stdout, "{}", text[0])?;
                    interpreter = Some(Interpreter::new(&output)?)
                }
            },
            "s" | "step" => match &mut interpreter {
                Some(i) => {
                    let position = i.step()?;
                    let line = line_map
                        .iter()
                        .position(|&offset| offset == position)
                        .unwrap();
                    writeln!(stdout, "{}", text[line])?;
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
