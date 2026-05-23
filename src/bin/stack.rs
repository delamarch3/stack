use std::env;
use std::fs::File;
use std::process;

use stack::interpreter::{Interpreter, InterpreterOptions};
use stack::locals::Locals;
use stack::output::Output;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let mut args = env::args();
    let program = args.next().unwrap();
    let Some(path) = args.next() else {
        eprintln!("usage: {} path/to/file [-- arg1 arg2]", program);
        process::exit(1);
    };

    let mut argv: Vec<*const u8> = Vec::new();
    match args.next().as_ref().map(String::as_str) {
        Some("--") => argv.extend(args.into_iter().map(|mut arg| {
            arg.push('\0');
            Box::leak(arg.into_boxed_str()).as_ptr()
        })),
        Some(arg) => {
            eprintln!("unknown argument: {}", arg);
            eprintln!("usage: {} path/to/file [-- arg1 arg2]", program);
            process::exit(1);
        }
        None => {}
    }

    let file = File::open(path)?;
    let output = Output::deserialise(file)?;

    let options = InterpreterOptions::default().with_argv(create_argv_locals(argv));

    let mut interpreter = Interpreter::new(&output, options)?;
    if let Err(err) = interpreter.run() {
        eprintln!("{err}");
    };

    println!("{}", interpreter.frames().last().unwrap().opstack);

    Ok(())
}

fn create_argv_locals(args: Vec<*const u8>) -> Locals {
    let args = Box::leak(args.into_boxed_slice());

    let mut locals = Locals::default();
    locals.write(0, args.len() as i32);
    locals.write(1, args.as_ptr() as u64);
    locals
}
