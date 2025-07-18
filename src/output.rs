use std::collections::HashMap;
use std::{io::Read, mem};

use crate::program::{Bytecode, Program};
use crate::{Number, Result};

#[derive(Debug)]
pub struct Output {
    labels: HashMap<u64, String>,
    entry: u64,
    data: Vec<u8>,
    text: Vec<u8>,
}

// TODO: Debugger - map position to line?

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const TAB_SPACES: usize = 4;

        fn fmt_with_operand<T: Number>(
            f: &mut std::fmt::Formatter<'_>,
            labels: &HashMap<u64, String>,
            pc: &mut Program<'_>,
            word: &str,
        ) -> std::fmt::Result {
            write!(f, "{} ", word)?;
            let operand = pc.next::<T>().map_err(|_| std::fmt::Error)?;
            write!(f, "{operand}")?;

            let b = operand.to_le_bytes();
            if b.as_ref().len() == 8 {
                let n = u64::from_le_bytes(b.as_ref().try_into().unwrap());
                if let Some(label) = labels.get(&n) {
                    write!(f, " ; {}", label)?;
                }
            }

            Ok(())
        }

        let next_position = |pc: &Program<'_>| {
            pc.position() + mem::size_of::<u64>() as u64 + self.data.len() as u64
        };

        // Write entry
        if let Some(entry) = self.labels.get(&self.entry) {
            writeln!(f, ".entry {}", entry)?;
        } else {
            writeln!(f, ".entry {}", self.entry)?;
        }
        writeln!(f)?;

        // Write data
        for (i, chunk) in self.data.as_slice().chunks(16).enumerate() {
            let pos = i + mem::size_of::<u64>();

            write!(f, "{pos}: ")?;
            for b in chunk {
                write!(f, "{:x} ", b)?;
            }

            write!(f, "| ")?;
            for b in chunk {
                if b.is_ascii_graphic() {
                    write!(f, "{}", *b as char)?
                } else {
                    write!(f, ".")?
                }
            }
            writeln!(f)?;
        }
        writeln!(f)?;

        // Write text
        let mut pc = Program::new(&self.text);
        let mut pos = next_position(&pc);
        while let Ok(op) = pc.next_op() {
            if let Some(label) = self.labels.get(&(pos)) {
                write!(f, "{label}:\n")?;
            }

            write!(f, "{:TAB_SPACES$}{}: ", "", pos)?;

            match op {
                Bytecode::Push => fmt_with_operand::<i32>(f, &self.labels, &mut pc, "push")?,
                Bytecode::PushD => fmt_with_operand::<i64>(f, &self.labels, &mut pc, "push.d")?,
                Bytecode::PushB => fmt_with_operand::<i8>(f, &self.labels, &mut pc, "push")?,
                Bytecode::Pop => write!(f, "pop")?,
                Bytecode::PopD => write!(f, "pop.d")?,
                Bytecode::PopB => write!(f, "pop.b")?,
                Bytecode::Load => fmt_with_operand::<u64>(f, &self.labels, &mut pc, "load")?,
                Bytecode::LoadD => fmt_with_operand::<u64>(f, &self.labels, &mut pc, "load.d")?,
                Bytecode::LoadB => fmt_with_operand::<u64>(f, &self.labels, &mut pc, "load.b")?,
                Bytecode::Store => fmt_with_operand::<u64>(f, &self.labels, &mut pc, "store")?,
                Bytecode::StoreD => fmt_with_operand::<u64>(f, &self.labels, &mut pc, "store.d")?,
                Bytecode::StoreB => fmt_with_operand::<u64>(f, &self.labels, &mut pc, "store.b")?,
                Bytecode::Get => fmt_with_operand::<u64>(f, &self.labels, &mut pc, "get")?,
                Bytecode::GetD => fmt_with_operand::<u64>(f, &self.labels, &mut pc, "get.d")?,
                Bytecode::GetB => fmt_with_operand::<u64>(f, &self.labels, &mut pc, "get.b")?,
                Bytecode::Add => write!(f, "add")?,
                Bytecode::AddD => write!(f, "add.d")?,
                Bytecode::AddB => write!(f, "add.b")?,
                Bytecode::Sub => write!(f, "sub")?,
                Bytecode::SubD => write!(f, "sub.d")?,
                Bytecode::SubB => write!(f, "sub.b")?,
                Bytecode::Mul => write!(f, "mul")?,
                Bytecode::MulD => write!(f, "mul.d")?,
                Bytecode::Div => write!(f, "div")?,
                Bytecode::DivD => write!(f, "div.d")?,
                Bytecode::Cmp => write!(f, "cmp")?,
                Bytecode::CmpD => write!(f, "cmp.d")?,
                Bytecode::Dup => write!(f, "dup")?,
                Bytecode::DupD => write!(f, "dup.d")?,
                Bytecode::Jmp => fmt_with_operand::<u64>(f, &self.labels, &mut pc, "jmp")?,
                Bytecode::JmpLt => fmt_with_operand::<u64>(f, &self.labels, &mut pc, "jmp.lt")?,
                Bytecode::JmpEq => fmt_with_operand::<u64>(f, &self.labels, &mut pc, "jmp.eq")?,
                Bytecode::JmpGt => fmt_with_operand::<u64>(f, &self.labels, &mut pc, "jmp.gt")?,
                Bytecode::JmpNe => fmt_with_operand::<u64>(f, &self.labels, &mut pc, "jmp.ne")?,
                Bytecode::Call => fmt_with_operand::<u64>(f, &self.labels, &mut pc, "call")?,
                Bytecode::Fail => write!(f, "fail")?,
                Bytecode::Ret => write!(f, "ret")?,
                Bytecode::RetW => write!(f, "ret.w")?,
                Bytecode::RetD => write!(f, "ret.d")?,
            }

            pos = next_position(&pc);
            writeln!(f)?;
        }

        Ok(())
    }
}

impl From<Output> for Vec<u8> {
    // TODO: file format
    fn from(output: Output) -> Self {
        let mut program =
            Vec::with_capacity(size_of::<usize>() + output.data.len() + output.text.len());
        program.extend(output.entry.to_le_bytes());
        program.extend(output.data);
        program.extend(output.text);
        program
    }
}

impl Output {
    pub fn new(entry: u64, labels: HashMap<u64, String>, data: Vec<u8>, text: Vec<u8>) -> Self {
        Self {
            entry,
            labels,
            data,
            text,
        }
    }

    pub fn from_reader<R: Read>(mut r: R) -> Result<Self> {
        let mut entry_buf = [0u8; mem::size_of::<u64>()];
        let n = r.read(&mut entry_buf)?;
        if n < mem::size_of::<u64>() {
            Err(format!("read less than expected bytes: {n}"))?;
        }
        let entry = u64::from_le_bytes(entry_buf);

        // TODO
        let labels = HashMap::new();

        // TODO
        let data = Vec::new();

        let mut text = Vec::new();
        r.read_to_end(&mut text)?;

        Ok(Self {
            entry,
            labels,
            data,
            text,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::assembler::Assembler;
    use crate::Result;

    #[test]
    fn test_display() -> Result<()> {
        let src = "
.entry main

.data record
    .string \"abc\"
    .byte 0
    .word 76

main:
    push.d record
    push 22
    push 33
    call add
    store 0
    ret

add:
   load 0
   load 1
   add
   ret";

        let output = Assembler::new(&src).assemble()?;
        let have = output.to_string();
        let want = "\
.entry main

8: 61 62 63 0 4c 0 0 0 | abc.L...

main:
    16: push.d 8 ; record
    25: push 22
    30: push 33
    35: call 54 ; add
    44: store 0
    53: ret
add:
    54: load 0
    63: load 1
    72: add
    73: ret
";

        assert_eq!(want, have);

        Ok(())
    }
}
