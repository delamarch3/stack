use std::env;
use std::fs::File;
use std::io::Read;
use std::process;

use stack::interpreter::Interpreter;
use stack::stack::Stack;

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

    let stack = Stack::new();
    let mut interpreter = Interpreter::new(stack, &src);
    interpreter.run().unwrap();

    eprintln!("{}", interpreter.stack());

    Ok(())
}
