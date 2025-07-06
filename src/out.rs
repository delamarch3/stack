use std::{io::Read, mem};

use crate::program::{Bytecode, Program};
use crate::{Number, Result};

pub struct StackOut {
    entry: u64,

    // TODO
    // data: Vec<u8>,
    // labels: HashMap<u64, String>,
    text: Vec<u8>,
}

// TODO: Interpreter takes StackOut
// TODO: Debugger - use StackOut.fmt and map position to line?

impl std::fmt::Display for StackOut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const TAB_SPACES: usize = 4;

        fn fmt_with_operand<T: Number>(
            f: &mut std::fmt::Formatter<'_>,
            pc: &mut Program<'_>,
            word: &str,
        ) -> std::fmt::Result {
            write!(f, "{} ", word)?;
            let operand = pc.next::<T>().map_err(|_| std::fmt::Error)?;
            write!(f, "{operand}")
        }

        writeln!(f, ".entry {}", self.entry)?;
        let mut pc = Program::new(&self.text);
        let mut pos = pc.position();
        while let Ok(op) = pc.next_op() {
            write!(f, "{:TAB_SPACES$}{}: ", "", pos + 8)?; // TODO: position shouldn't count entry
            match op {
                Bytecode::Push => fmt_with_operand::<i32>(f, &mut pc, "push")?,
                Bytecode::PushD => fmt_with_operand::<i64>(f, &mut pc, "push.d")?,
                Bytecode::PushB => fmt_with_operand::<i8>(f, &mut pc, "push")?,
                Bytecode::Pop => write!(f, "pop")?,
                Bytecode::PopD => write!(f, "pop.d")?,
                Bytecode::PopB => write!(f, "pop.b")?,
                Bytecode::Load => fmt_with_operand::<u64>(f, &mut pc, "load")?,
                Bytecode::LoadD => fmt_with_operand::<u64>(f, &mut pc, "load.d")?,
                Bytecode::LoadB => fmt_with_operand::<u64>(f, &mut pc, "load.b")?,
                Bytecode::Store => fmt_with_operand::<u64>(f, &mut pc, "store")?,
                Bytecode::StoreD => fmt_with_operand::<u64>(f, &mut pc, "store.d")?,
                Bytecode::StoreB => fmt_with_operand::<u64>(f, &mut pc, "store.b")?,
                Bytecode::Get => fmt_with_operand::<u64>(f, &mut pc, "get")?,
                Bytecode::GetD => fmt_with_operand::<u64>(f, &mut pc, "get.d")?,
                Bytecode::GetB => fmt_with_operand::<u64>(f, &mut pc, "get.b")?,
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
                Bytecode::Jmp => fmt_with_operand::<u64>(f, &mut pc, "jmp")?,
                Bytecode::JmpLt => fmt_with_operand::<u64>(f, &mut pc, "jmp.lt")?,
                Bytecode::JmpEq => fmt_with_operand::<u64>(f, &mut pc, "jmp.eq")?,
                Bytecode::JmpGt => fmt_with_operand::<u64>(f, &mut pc, "jmp.gt")?,
                Bytecode::JmpNe => fmt_with_operand::<u64>(f, &mut pc, "jmp.ne")?,
                Bytecode::Call => fmt_with_operand::<u64>(f, &mut pc, "call")?,
                Bytecode::Fail => write!(f, "fail")?,
                Bytecode::Ret => write!(f, "ret")?,
                Bytecode::RetW => write!(f, "ret.w")?,
                Bytecode::RetD => write!(f, "ret.d")?,
            }

            writeln!(f)?;
            pos = pc.position();
        }

        Ok(())
    }
}

impl StackOut {
    pub fn from_reader<R: Read>(mut r: R) -> Result<Self> {
        let mut entry_buf = [0u8; mem::size_of::<u64>()];
        let n = r.read(&mut entry_buf)?;
        if n < mem::size_of::<u64>() {
            Err(format!("read less than expected bytes: {n}"))?;
        }
        let entry = u64::from_le_bytes(entry_buf);

        let mut text = Vec::new();
        r.read_to_end(&mut text)?;

        Ok(Self { entry, text })
    }
}

#[cfg(test)]
mod test {
    use crate::assembler::Assembler;
    use crate::Result;

    use super::StackOut;

    #[test]
    fn test_display() -> Result<()> {
        let src = "
.entry main

main:
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

        let program = Assembler::new(&src).assemble()?;
        let stack_file = StackOut::from_reader(program.as_slice())?;
        let have = stack_file.to_string();
        let want = "\
.entry 8
    8: push 22
    13: push 33
    18: call 37
    27: store 0
    36: ret
    37: load 0
    46: load 1
    55: add
    56: ret
";

        assert_eq!(want, have);

        Ok(())
    }
}
