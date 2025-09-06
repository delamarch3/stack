use crate::{Number, Result};
use std::io::{Cursor, Read};

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Bytecode {
    Add,
    AddB,
    AddD,
    Alloc,
    Cmp,
    CmpD,
    DataPtr,
    Div,
    DivD,
    Dup,
    DupD,
    Get,
    GetB,
    GetD,
    Jmp,
    JmpEq,
    JmpGt,
    JmpLt,
    JmpNe,
    Load,
    LoadB,
    LoadD,
    Mul,
    MulD,
    Ptr,
    Pop,
    PopB,
    PopD,
    Push,
    PushB,
    PushD,
    Read,
    Store,
    StoreB,
    StoreD,
    Sub,
    SubB,
    SubD,
    System,
    Write,

    Call,
    Panic,
    Ret,
    RetW,
    RetD,
}

impl std::fmt::Display for Bytecode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let width = f.width().unwrap_or_default();

        match self {
            Bytecode::Add => write!(f, "{:width$}", "add"),
            Bytecode::AddB => write!(f, "{:width$}", "add.b"),
            Bytecode::AddD => write!(f, "{:width$}", "add.d"),
            Bytecode::Alloc => write!(f, "{:width$}", "alloc"),
            Bytecode::Cmp => write!(f, "{:width$}", "cmp"),
            Bytecode::CmpD => write!(f, "{:width$}", "cmp.d"),
            Bytecode::DataPtr => write!(f, "{:width$}", "data"),
            Bytecode::Div => write!(f, "{:width$}", "div"),
            Bytecode::DivD => write!(f, "{:width$}", "div.d"),
            Bytecode::Dup => write!(f, "{:width$}", "dup"),
            Bytecode::DupD => write!(f, "{:width$}", "dup.d"),
            Bytecode::Get => write!(f, "{:width$}", "get"),
            Bytecode::GetB => write!(f, "{:width$}", "get.b"),
            Bytecode::GetD => write!(f, "{:width$}", "get.d"),
            Bytecode::Jmp => write!(f, "{:width$}", "jmp"),
            Bytecode::JmpEq => write!(f, "{:width$}", "jmp.eq"),
            Bytecode::JmpGt => write!(f, "{:width$}", "jmp.gt"),
            Bytecode::JmpLt => write!(f, "{:width$}", "jmp.lt"),
            Bytecode::JmpNe => write!(f, "{:width$}", "jmp.ne"),
            Bytecode::Load => write!(f, "{:width$}", "load"),
            Bytecode::LoadB => write!(f, "{:width$}", "load.b"),
            Bytecode::LoadD => write!(f, "{:width$}", "load.d"),
            Bytecode::Mul => write!(f, "{:width$}", "mul"),
            Bytecode::MulD => write!(f, "{:width$}", "mul.d"),
            Bytecode::Ptr => write!(f, "{:width$}", "ptr"),
            Bytecode::Pop => write!(f, "{:width$}", "pop"),
            Bytecode::PopB => write!(f, "{:width$}", "pop.b"),
            Bytecode::PopD => write!(f, "{:width$}", "pop.d"),
            Bytecode::Push => write!(f, "{:width$}", "push"),
            Bytecode::PushB => write!(f, "{:width$}", "push.b"),
            Bytecode::PushD => write!(f, "{:width$}", "push.d"),
            Bytecode::Read => write!(f, "{:width$}", "read"),
            Bytecode::Store => write!(f, "{:width$}", "store"),
            Bytecode::StoreB => write!(f, "{:width$}", "store.b"),
            Bytecode::StoreD => write!(f, "{:width$}", "store.d"),
            Bytecode::Sub => write!(f, "{:width$}", "sub"),
            Bytecode::SubB => write!(f, "{:width$}", "sub.b"),
            Bytecode::SubD => write!(f, "{:width$}", "sub.d"),
            Bytecode::System => write!(f, "{:width$}", "system"),
            Bytecode::Write => write!(f, "{:width$}", "write"),

            Bytecode::Call => write!(f, "{:width$}", "call"),
            Bytecode::Panic => write!(f, "{:width$}", "panic"),
            Bytecode::Ret => write!(f, "{:width$}", "ret"),
            Bytecode::RetW => write!(f, "{:width$}", "ret.w"),
            Bytecode::RetD => write!(f, "{:width$}", "ret.d"),
        }
    }
}

#[derive(Clone)]
pub struct Program<T: AsRef<[u8]>> {
    counter: Cursor<T>,
}

impl<T: AsRef<[u8]>> Program<T> {
    pub fn new(src: T) -> Self {
        let counter = Cursor::new(src);
        Self { counter }
    }

    pub fn set_position(&mut self, position: u64) {
        self.counter.set_position(position);
    }

    pub fn position(&self) -> u64 {
        self.counter.position()
    }

    pub fn next<N: Number>(&mut self) -> Result<N> {
        let mut buf = [0u8; 8];
        let n = self.counter.read(&mut buf[0..N::SIZE])?;
        if n == 0 {
            Err("unexpected end of program")?;
        }
        if n < N::SIZE {
            Err(format!("read less than expected bytes: {n}"))?;
        }

        Ok(N::from_le_bytes(&buf[0..N::SIZE]))
    }

    pub fn next_op(&mut self) -> Result<Bytecode> {
        let op = self.next::<u8>()?;
        assert!(op <= Bytecode::RetD as u8);
        let op = unsafe { std::mem::transmute::<u8, Bytecode>(op) };
        Ok(op)
    }

    pub fn get<N: Number>(&mut self, offset: usize) -> N {
        N::from_le_bytes(&self.counter.get_ref().as_ref()[offset..offset + N::SIZE])
    }

    pub fn getptr(&mut self, offset: usize) -> *const u8 {
        self.counter.get_ref().as_ref()[offset..].as_ptr()
    }
}
