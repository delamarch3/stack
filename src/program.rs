use crate::{Number, Result};
use std::io::{Cursor, Read};

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Bytecode {
    ALoad,
    ALoadB,
    ALoadD,
    AStore,
    AStoreB,
    AStoreD,
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
    Free,
    Get,
    GetB,
    GetD,
    Jmp,
    JmpEq,
    JmpGe,
    JmpGt,
    JmpLe,
    JmpLt,
    JmpNe,
    Load,
    LoadB,
    LoadD,
    Mul,
    MulD,
    Pop,
    PopB,
    PopD,
    Push,
    PushB,
    PushD,
    Store,
    StoreB,
    StoreD,
    Sub,
    SubB,
    SubD,
    System,

    Call,
    Panic,
    Ret,
    RetW,
    RetD,
}

impl std::fmt::Display for Bytecode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Bytecode::ALoad => "aload".fmt(f),
            Bytecode::ALoadB => "aload.b".fmt(f),
            Bytecode::ALoadD => "aload.d".fmt(f),
            Bytecode::AStore => "astore".fmt(f),
            Bytecode::AStoreB => "astore.b".fmt(f),
            Bytecode::AStoreD => "astore.d".fmt(f),
            Bytecode::Add => "add".fmt(f),
            Bytecode::AddB => "add.b".fmt(f),
            Bytecode::AddD => "add.d".fmt(f),
            Bytecode::Alloc => "alloc".fmt(f),
            Bytecode::Cmp => "cmp".fmt(f),
            Bytecode::CmpD => "cmp.d".fmt(f),
            Bytecode::DataPtr => "dataptr".fmt(f),
            Bytecode::Div => "div".fmt(f),
            Bytecode::DivD => "div.d".fmt(f),
            Bytecode::Dup => "dup".fmt(f),
            Bytecode::DupD => "dup.d".fmt(f),
            Bytecode::Free => "free".fmt(f),
            Bytecode::Get => "get".fmt(f),
            Bytecode::GetB => "get.b".fmt(f),
            Bytecode::GetD => "get.d".fmt(f),
            Bytecode::Jmp => "jmp".fmt(f),
            Bytecode::JmpEq => "jmp.eq".fmt(f),
            Bytecode::JmpGe => "jmp.ge".fmt(f),
            Bytecode::JmpGt => "jmp.gt".fmt(f),
            Bytecode::JmpLe => "jmp.le".fmt(f),
            Bytecode::JmpLt => "jmp.lt".fmt(f),
            Bytecode::JmpNe => "jmp.ne".fmt(f),
            Bytecode::Load => "load".fmt(f),
            Bytecode::LoadB => "load.b".fmt(f),
            Bytecode::LoadD => "load.d".fmt(f),
            Bytecode::Mul => "mul".fmt(f),
            Bytecode::MulD => "mul.d".fmt(f),
            Bytecode::Pop => "pop".fmt(f),
            Bytecode::PopB => "pop.b".fmt(f),
            Bytecode::PopD => "pop.d".fmt(f),
            Bytecode::Push => "push".fmt(f),
            Bytecode::PushB => "push.b".fmt(f),
            Bytecode::PushD => "push.d".fmt(f),
            Bytecode::Store => "store".fmt(f),
            Bytecode::StoreB => "store.b".fmt(f),
            Bytecode::StoreD => "store.d".fmt(f),
            Bytecode::Sub => "sub".fmt(f),
            Bytecode::SubB => "sub.b".fmt(f),
            Bytecode::SubD => "sub.d".fmt(f),
            Bytecode::System => "system".fmt(f),

            Bytecode::Call => "call".fmt(f),
            Bytecode::Panic => "panic".fmt(f),
            Bytecode::Ret => "ret".fmt(f),
            Bytecode::RetW => "ret.w".fmt(f),
            Bytecode::RetD => "ret.d".fmt(f),
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
        assert!(
            op <= Bytecode::RetD as u8,
            "unexpected opcode: {op} at {position}",
            position = self.counter.position()
        );
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
