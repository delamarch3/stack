use crate::{Number, Result};
use std::io::{Cursor, Read};

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Bytecode {
    Push,
    PushD,
    PushB,
    Pop,
    PopD,
    PopB,
    Load,
    LoadD,
    LoadB,
    Store,
    StoreD,
    StoreB,
    Get,
    GetD,
    GetB,
    Add,
    AddD,
    AddB,
    Sub,
    SubD,
    SubB,
    Mul,
    MulD,
    Div,
    DivD,
    Cmp,
    CmpD,
    Dup,
    DupD,

    Jmp,
    JmpLt,
    JmpEq,
    JmpGt,
    JmpNe,
    Call,
    Panic,
    Ret,
    RetW,
    RetD,
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
}
