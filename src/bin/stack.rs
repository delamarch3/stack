use std::env;
use std::fs::File;
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

    let mut interpreter = Interpreter::new(&output)?;
    if let Err(err) = interpreter.run() {
        eprintln!("{err}");
    };

    interpreter.print_opstack();

    Ok(())
}
