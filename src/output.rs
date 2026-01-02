use std::collections::HashMap;
use std::fmt::Write;
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

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_entry(f).map_err(|_| std::fmt::Error)?;
        writeln!(f)?;

        self.fmt_data(f).map_err(|_| std::fmt::Error)?;
        writeln!(f)?;

        self.fmt_text(f).map_err(|_| std::fmt::Error)?;

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

    pub fn labels(&self) -> &HashMap<u64, String> {
        &self.labels
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

        let mut output = Vec::with_capacity(
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
        output.extend(self.entry.to_le_bytes());

        // Data and text
        output.extend(u16::try_from(self.data.len()).unwrap().to_le_bytes());
        output.extend(&self.data);
        output.extend(u16::try_from(self.text.len()).unwrap().to_le_bytes());
        output.extend(&self.text);

        // Label offsets
        output.extend(u16::try_from(offsets.len()).unwrap().to_le_bytes());
        offsets
            .into_iter()
            .for_each(|offset| output.extend(offset.to_le_bytes()));

        // Label values
        output.extend(u16::try_from(labels.len()).unwrap().to_le_bytes());
        labels.into_iter().for_each(|label| {
            output.extend(u16::try_from(label.len()).unwrap().to_le_bytes());
            output.extend(label.as_bytes());
        });

        output
    }

    pub fn fmt_entry(&self, f: &mut impl Write) -> Result<()> {
        if let Some(entry) = self.labels.get(&self.entry) {
            writeln!(f, ".entry {}", entry)?;
        } else {
            writeln!(f, ".entry {}", self.entry)?;
        }

        Ok(())
    }

    pub fn fmt_data(&self, f: &mut impl Write) -> Result<()> {
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

        Ok(())
    }

    pub fn fmt_text(&self, f: &mut impl Write) -> Result<HashMap<u64, usize>> {
        const POS_WIDTH: usize = 4;
        const INST_WIDTH: usize = 7;
        const OP_WIDTH: usize = 4;

        fn fmt_with_operand<T: Number>(
            f: &mut impl Write,
            pc: &mut Program<&[u8]>,
            labels: &HashMap<u64, String>,
            op: Bytecode,
        ) -> std::fmt::Result {
            write!(f, "{op:INST_WIDTH$}")?;
            let operand = pc.next::<T>().map_err(|_| std::fmt::Error)?;
            write!(f, "{operand:OP_WIDTH$}")?;

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

        // Write text
        let mut line = 0;
        let mut lines = HashMap::new(); // Position -> Line
        let mut pc = Program::new(self.text.as_slice());
        let mut pos = next_position(&pc);
        lines.insert(pos, line);
        while let Ok(op) = pc.next_op() {
            if let Some(label) = self.labels.get(&pos) {
                writeln!(f, "{label}:")?;
                line += 1;
            }

            lines.insert(pos, line);
            write!(f, "{pos:POS_WIDTH$}: ")?;

            match op {
                Bytecode::Call => fmt_with_operand::<u64>(f, &mut pc, &self.labels, op)?,
                Bytecode::DataPtr => fmt_with_operand::<u64>(f, &mut pc, &self.labels, op)?,
                Bytecode::Jmp => fmt_with_operand::<u64>(f, &mut pc, &self.labels, op)?,
                Bytecode::JmpEq => fmt_with_operand::<u64>(f, &mut pc, &self.labels, op)?,
                Bytecode::JmpGe => fmt_with_operand::<u64>(f, &mut pc, &self.labels, op)?,
                Bytecode::JmpGt => fmt_with_operand::<u64>(f, &mut pc, &self.labels, op)?,
                Bytecode::JmpLe => fmt_with_operand::<u64>(f, &mut pc, &self.labels, op)?,
                Bytecode::JmpLt => fmt_with_operand::<u64>(f, &mut pc, &self.labels, op)?,
                Bytecode::JmpNe => fmt_with_operand::<u64>(f, &mut pc, &self.labels, op)?,
                Bytecode::Load => fmt_with_operand::<u64>(f, &mut pc, &self.labels, op)?,
                Bytecode::LoadB => fmt_with_operand::<u64>(f, &mut pc, &self.labels, op)?,
                Bytecode::LoadD => fmt_with_operand::<u64>(f, &mut pc, &self.labels, op)?,
                Bytecode::Push => fmt_with_operand::<i32>(f, &mut pc, &self.labels, op)?,
                Bytecode::PushB => fmt_with_operand::<i8>(f, &mut pc, &self.labels, op)?,
                Bytecode::PushD => fmt_with_operand::<i64>(f, &mut pc, &self.labels, op)?,
                Bytecode::Store => fmt_with_operand::<u64>(f, &mut pc, &self.labels, op)?,
                Bytecode::StoreB => fmt_with_operand::<u64>(f, &mut pc, &self.labels, op)?,
                Bytecode::StoreD => fmt_with_operand::<u64>(f, &mut pc, &self.labels, op)?,

                Bytecode::ALoad
                | Bytecode::ALoadB
                | Bytecode::ALoadD
                | Bytecode::AStore
                | Bytecode::AStoreB
                | Bytecode::AStoreD
                | Bytecode::Add
                | Bytecode::AddB
                | Bytecode::AddD
                | Bytecode::Alloc
                | Bytecode::Cmp
                | Bytecode::CmpD
                | Bytecode::Div
                | Bytecode::DivD
                | Bytecode::Dup
                | Bytecode::DupD
                | Bytecode::Free
                | Bytecode::Get
                | Bytecode::GetB
                | Bytecode::GetD
                | Bytecode::Mul
                | Bytecode::MulD
                | Bytecode::Pop
                | Bytecode::PopB
                | Bytecode::PopD
                | Bytecode::Sub
                | Bytecode::SubB
                | Bytecode::SubD
                | Bytecode::System
                | Bytecode::Panic
                | Bytecode::Ret
                | Bytecode::RetW
                | Bytecode::RetD => write!(f, "{op}")?,
            }

            pos = next_position(&pc);
            line += 1;
            writeln!(f)?;
        }

        Ok(lines)
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

        let output = Assembler::new().assemble(src)?;
        let have = output.to_string();
        let want = "\
.entry main

8: 61 62 63 00 4c 00 00 00 |abc.L...|

main:
  16: push.d    8 ; record
  25: push     22
  30: push     33
  35: call     54 ; add
  44: store     0
  53: ret
add:
  54: load      0
  63: load      1
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
        let want = Assembler::new().assemble(src)?;
        let serialised = want.clone().serialise();
        let have = Output::deserialise(serialised.as_slice())?;

        assert_eq!(want, have);

        Ok(())
    }
}
