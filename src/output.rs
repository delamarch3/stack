use std::collections::HashMap;
use std::io::Read;

use crate::program::{Bytecode, Program};
use crate::{Bytes, Number, Result};

#[derive(Debug, Clone, PartialEq)]
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
            pc: &mut Program<&[u8]>,
            word: &str,
        ) -> std::fmt::Result {
            write!(f, "{} ", word)?;
            let operand = pc.next::<T>().map_err(|_| std::fmt::Error)?;
            write!(f, "{operand}")?;

            // Check if the operand is also a label offset. It may not be so it is not directly
            // substituted
            if let Ok(offset) =
                <[u8; 8]>::try_from(operand.to_le_bytes().as_ref()).map(u64::from_le_bytes)
            {
                if let Some(label) = labels.get(&offset) {
                    write!(f, " ; {}", label)?;
                }
            }

            Ok(())
        }

        let next_position =
            |pc: &Program<&[u8]>| pc.position() + size_of::<u64>() as u64 + self.data.len() as u64;

        // Write entry
        if let Some(entry) = self.labels.get(&self.entry) {
            writeln!(f, ".entry {}", entry)?;
        } else {
            writeln!(f, ".entry {}", self.entry)?;
        }
        writeln!(f)?;

        // Write data
        for (i, chunk) in self.data.as_slice().chunks(16).enumerate() {
            let pos = i + size_of::<u64>();

            write!(f, "{pos}: ")?;
            for b in chunk {
                write!(f, "{:02x} ", b)?;
            }

            write!(f, "|")?;
            for b in chunk {
                if b.is_ascii_graphic() {
                    write!(f, "{}", *b as char)?
                } else {
                    write!(f, ".")?
                }
            }
            writeln!(f, "|")?;
        }
        writeln!(f)?;

        // Write text
        let mut pc = Program::new(self.text.as_slice());
        let mut pos = next_position(&pc);
        while let Ok(op) = pc.next_op() {
            if let Some(label) = self.labels.get(&(pos)) {
                writeln!(f, "{label}:")?;
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

impl From<&Output> for Vec<u8> {
    fn from(output: &Output) -> Self {
        let mut program =
            Vec::with_capacity(size_of::<u64>() + output.data.len() + output.text.len());
        program.extend(output.entry.to_le_bytes());
        program.extend(&output.data);
        program.extend(&output.text);
        program
    }
}

impl From<Output> for Vec<u8> {
    fn from(output: Output) -> Self {
        (&output).into()
    }
}

impl Output {
    pub fn new(entry: u64, data: Vec<u8>, text: Vec<u8>, labels: HashMap<u64, String>) -> Self {
        Self {
            entry,
            data,
            text,
            labels,
        }
    }

    pub fn deserialise<R: Read>(mut r: R) -> Result<Self> {
        let entry = r.read_u64()?;

        // Data and text
        let len = r.read_u16()?;
        let data = r.read_n(len as usize)?;
        let len = r.read_u16()?;
        let text = r.read_n(len as usize)?;

        // Label offsets
        let len = r.read_u16()?;
        let mut offsets: Vec<u64> = Vec::new();
        for _ in 0..len {
            let offset = r.read_u64()?;
            offsets.push(offset);
        }

        // Label values
        let len = r.read_u16()?;
        let mut labels: Vec<String> = Vec::new();
        for _ in 0..len {
            let len = r.read_u16()?;
            let data = r.read_n(len as usize)?;
            let label = String::from_utf8(data)?;
            labels.push(label);
        }

        assert!(offsets.len() == labels.len());
        let labels = std::iter::zip(offsets, labels).collect::<HashMap<u64, String>>();

        Ok(Self {
            labels,
            entry,
            data,
            text,
        })
    }

    pub fn serialise(self) -> Vec<u8> {
        let (offsets, labels) = self.labels.into_iter().collect::<(Vec<u64>, Vec<String>)>();

        let mut program = Vec::with_capacity(
            size_of::<u64>() // entry
                + size_of::<u16>() // data
                + self.data.len()
                + size_of::<u16>() // text
                + self.text.len()
                + size_of::<u16>() // offsets
                + (offsets.len() * size_of::<u64>())
                + size_of::<u16>() // labels (each as [length|data])
                + (labels.len() * size_of::<u16>()) + labels.iter().fold(0, |acc, l| acc + l.len()),
        );

        // Entry
        program.extend(self.entry.to_le_bytes());

        // Data and text
        program.extend(u16::try_from(self.data.len()).unwrap().to_le_bytes());
        program.extend(&self.data);
        program.extend(u16::try_from(self.text.len()).unwrap().to_le_bytes());
        program.extend(&self.text);

        // Label offsets
        program.extend(u16::try_from(offsets.len()).unwrap().to_le_bytes());
        offsets
            .into_iter()
            .for_each(|offset| program.extend(offset.to_le_bytes()));

        // Label values
        program.extend(u16::try_from(labels.len()).unwrap().to_le_bytes());
        labels.into_iter().for_each(|label| {
            program.extend(u16::try_from(label.len()).unwrap().to_le_bytes());
            program.extend(label.as_bytes());
        });

        program
    }
}

#[cfg(test)]
mod test {
    use crate::assembler::Assembler;
    use crate::Result;

    use super::Output;

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

8: 61 62 63 00 4c 00 00 00 |abc.L...|

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

    #[test]
    fn test_serde_roundtrip() -> Result<()> {
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
        let want = Assembler::new(&src).assemble()?;
        let serialised = want.clone().serialise();
        let have = Output::deserialise(serialised.as_slice())?;

        assert_eq!(want, have);

        Ok(())
    }
}
