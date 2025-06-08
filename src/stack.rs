use std::mem;

use crate::Number;

const STACK_SIZE: usize = 512;
pub(crate) struct OperandStack {
    stack: [u8; STACK_SIZE],
    ptr: usize,
}

impl Default for OperandStack {
    fn default() -> Self {
        let stack = [0; STACK_SIZE];
        let ptr = 0;
        Self { stack, ptr }
    }
}

impl std::fmt::Display for OperandStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let from = self.ptr.saturating_sub(i32::SIZE * 8);
        let until = from + mem::size_of::<i32>() * 8;

        let width = 8;
        let mut sep = "";
        let mut slice = &self.stack[from..until];
        write!(f, "[")?;
        while slice.len() > 0 {
            let n = i32::from_le_bytes(slice[..i32::SIZE].try_into().unwrap());
            slice = &slice[i32::SIZE..];
            write!(f, "{sep}")?;
            write!(f, "{n:width$}")?;
            sep = ",";
        }
        write!(f, "]\n")?;

        let ptr = (self.ptr / 4).min(8);
        let width = ptr + ptr * width;
        write!(f, "{:width$}^", "")
    }
}

impl OperandStack {
    pub fn as_slice(&self) -> &[u8] {
        &self.stack[..self.ptr]
    }

    pub fn clear(&mut self) {
        self.ptr = 0;
    }

    pub fn push<T: Number>(&mut self, value: T) {
        self.stack[self.ptr..self.ptr + T::SIZE].copy_from_slice(value.to_le_bytes().as_ref());
        self.ptr += T::SIZE;
    }

    pub fn pop<T: Number>(&mut self) -> T {
        self.ptr -= T::SIZE;
        T::from_le_bytes(&self.stack[self.ptr..self.ptr + T::SIZE])
    }

    pub fn add<T: Number>(&mut self) {
        let (a, b) = (self.pop::<T>(), self.pop::<T>());
        let value = a + b;
        self.push(value);
    }

    pub fn sub<T: Number>(&mut self) {
        let (a, b) = (self.pop::<T>(), self.pop::<T>());
        let value = b - a;
        self.push(value);
    }

    pub fn mul<T: Number>(&mut self) {
        let (a, b) = (self.pop::<T>(), self.pop::<T>());
        let value = a * b;
        self.push(value);
    }

    pub fn div<T: Number>(&mut self) {
        let (a, b) = (self.pop::<T>(), self.pop::<T>());
        let value = b / a;
        self.push(value);
    }

    pub fn cmp<T: Number>(&mut self, lhs: T) {
        let rhs = self.pop::<T>();
        self.push(rhs.cmp(&lhs) as i32);
    }

    pub fn dup<T: Number>(&mut self) {
        let value = T::from_le_bytes(&self.stack[self.ptr - T::SIZE..self.ptr]);
        self.push(value);
    }
}

#[cfg(test)]
mod test {
    use super::OperandStack;

    #[test]
    fn test_stack() {
        let mut stack = OperandStack::default();
        stack.push(10);
        stack.push(15);
        stack.add::<i32>();
        assert_eq!(stack.pop::<i32>(), 25);

        stack.push::<i32>(0x40000000);
        stack.dup::<i32>();
        assert_eq!(stack.pop::<i64>(), 0x4000000040000000);
    }
}
