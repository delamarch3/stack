use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::process;

use stack::assembler::Assembler;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let mut args = env::args();
    let program = args.next().unwrap();
    let Some(path) = args.next() else {
        eprintln!("usage: {} path/to/file", program);
        process::exit(1);
    };

    let mut src = String::new();
    let mut file = File::open(path)?;
    file.read_to_string(&mut src)?;

    const OUTPUT_FILE: &str = "a.out";
    let output = Assembler::new().assemble(&src)?;
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(OUTPUT_FILE)?
        .write_all(&output.serialise())?;

    Ok(())
}
