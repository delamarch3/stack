use std::sync::{Arc, Mutex};

pub mod assembler;
pub mod debugger;
mod frame;
mod heap;
pub mod interpreter;
mod locals;
pub mod output;
mod program;
mod stack;
mod tokeniser;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub type SharedWriter = Arc<Mutex<dyn std::io::Write>>;

#[allow(dead_code)]
pub trait Number:
    Sized
    + Default
    + Ord
    + std::fmt::Debug
    + std::fmt::Display
    + std::str::FromStr
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::Div<Output = Self>
{
    const SIZE: usize;
    type Bytes: IntoIterator<Item = u8> + AsRef<[u8]> + AsMut<[u8]>;
    fn to_be_bytes(&self) -> Self::Bytes;
    fn to_le_bytes(&self) -> Self::Bytes;
    fn from_le_bytes(bytes: &[u8]) -> Self;
    fn from_be_bytes(bytes: &[u8]) -> Self;
}

macro_rules! impl_number {
    ($($ty:ty),*) => {
        $(
        impl Number for $ty {
            const SIZE: usize = std::mem::size_of::<$ty>();
            type Bytes = [u8; Self::SIZE];

            fn to_be_bytes(&self) -> Self::Bytes {
                <$ty>::to_be_bytes(*self)
            }

            fn to_le_bytes(&self) -> Self::Bytes {
                <$ty>::to_le_bytes(*self)
            }

            fn from_le_bytes(bytes: &[u8]) -> Self {
                <$ty>::from_le_bytes(bytes.try_into().unwrap())
            }

            fn from_be_bytes(bytes: &[u8]) -> Self {
                <$ty>::from_be_bytes(bytes.try_into().unwrap())
            }
        }
        )*
    };
}

impl_number!(u8, i8, i16, i32, i64, u64);

pub trait Bytes {
    fn read_u64(&mut self) -> Result<u64>;
    fn read_u16(&mut self) -> Result<u16>;
    fn read_n(&mut self, n: usize) -> Result<Vec<u8>>;
}

impl<T> Bytes for T
where
    T: std::io::Read,
{
    fn read_u64(&mut self) -> Result<u64> {
        let mut buf = [0u8; size_of::<u64>()];
        let n = self.read(&mut buf)?;
        if n < size_of::<u64>() {
            Err(format!("read less than expected bytes: {n}"))?;
        }

        Ok(u64::from_le_bytes(buf))
    }

    fn read_u16(&mut self) -> Result<u16> {
        let mut buf = [0u8; size_of::<u16>()];
        let n = self.read(&mut buf)?;
        if n < size_of::<u16>() {
            Err(format!("read less than expected bytes: {n}"))?;
        }

        Ok(u16::from_le_bytes(buf))
    }

    fn read_n(&mut self, n: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0; n];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }
}
