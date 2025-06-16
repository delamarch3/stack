use crate::Number;

const SLOT_SIZE: usize = std::mem::size_of::<i32>();
macro_rules! slot {
    ($ty:ty, $i:expr) => {{
        let from = $i * SLOT_SIZE;
        let to = from + std::mem::size_of::<$ty>();
        from..to
    }};
}

const LOCALS_SIZE: usize = std::mem::size_of::<i32>() * 128;
pub struct Locals {
    locals: Box<[u8; LOCALS_SIZE]>,
}

impl Default for Locals {
    fn default() -> Self {
        let locals = Box::new([0u8; LOCALS_SIZE]);
        Self { locals }
    }
}

impl Locals {
    pub fn read<T: Number>(&self, i: u64) -> T {
        T::from_le_bytes(&self.locals[slot!(T, i as usize)])
    }

    pub fn write<T: Number>(&mut self, i: u64, value: T) {
        self.locals[slot!(T, i as usize)].copy_from_slice(value.to_le_bytes().as_ref());
    }

    pub fn copy_from_slice(&mut self, slice: &[u8]) {
        self.locals[..slice.len()].copy_from_slice(slice);
    }
}
