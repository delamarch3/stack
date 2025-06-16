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
    Fail,
    Ret,
    RetW,
    RetD,
}

#[derive(Clone)]
pub struct Program<'a> {
    src: &'a [u8],
    counter: Cursor<&'a [u8]>,
}

impl<'a> Program<'a> {
    pub fn new(src: &'a [u8]) -> Self {
        let counter = Cursor::new(src);
        Self { src, counter }
    }

    pub fn set_position(&mut self, position: u64) {
        self.counter.set_position(position);
    }

    pub fn position(&self) -> u64 {
        self.counter.position()
    }

    pub fn next<T: Number>(&mut self) -> Result<T> {
        let mut buf = [0u8; 8];
        let n = self.counter.read(&mut buf[0..T::SIZE])?;
        if n < T::SIZE {
            Err(format!("read less than expected bytes: {n}"))?;
        }

        Ok(T::from_le_bytes(&buf[0..T::SIZE]))
    }

    pub fn next_op(&mut self) -> Result<Bytecode> {
        let op = self.next::<u8>()?;
        assert!(op <= Bytecode::RetD as u8);
        let op = unsafe { std::mem::transmute::<u8, Bytecode>(op) };
        Ok(op)
    }

    pub fn get<T: Number>(&mut self, offset: usize) -> T {
        T::from_le_bytes(&self.src[offset..offset + T::SIZE])
    }
}
