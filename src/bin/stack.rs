use std::env;
use std::fs::File;
use std::io::Read;
use std::process;

use stack::interpreter::Interpreter;
use stack::stack::OperandStack;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let mut args = env::args();
    let program = args.next().unwrap();
    let Some(path) = args.next() else {
        eprintln!("usage: {} path/to/file", program);
        process::exit(1);
    };

    let mut src = Vec::new();
    let mut file = File::open(path)?;
    file.read_to_end(&mut src)?;

    let stack = OperandStack::default();
    let mut interpreter = Interpreter::new(stack, &src);

    if let Err(err) = interpreter.run() {
        eprintln!("{err}");
    };

    eprintln!("{}", interpreter.opstack());

    Ok(())
}
