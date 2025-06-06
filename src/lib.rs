pub mod assembler;
pub mod expr;
pub mod interpreter;
pub mod stack;

#[allow(dead_code)]
trait Number {
    const SIZE: usize;
    type Bytes: IntoIterator<Item = u8> + AsRef<[u8]>;
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

impl_number!(i8, i16, i32, i64, usize);
