use crate::Number;

const STACK_SIZE: usize = 512;
const SLOT_SIZE: usize = std::mem::size_of::<i32>();
pub(crate) struct OperandStack {
    stack: [u8; STACK_SIZE],
    idx: usize,
}

impl Default for OperandStack {
    fn default() -> Self {
        let stack = [0; STACK_SIZE];
        let idx = 0;
        Self { stack, idx }
    }
}

impl std::fmt::Display for OperandStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let from = self.idx.saturating_sub(8) * SLOT_SIZE;
        let until = (from + 8) * SLOT_SIZE;

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

        let idx = self.idx;
        let min_idx = self.idx.min(8);
        let cursor = min_idx + min_idx * width;
        write!(f, "{:cursor$}^{idx}", "")
    }
}

impl OperandStack {
    pub fn as_slice(&self) -> &[u8] {
        &self.stack[..self.idx * SLOT_SIZE]
    }

    pub fn clear(&mut self) {
        self.idx = 0;
    }

    pub fn push<T: Number>(&mut self, value: T) {
        let offset = self.idx * SLOT_SIZE;
        self.idx += T::SIZE.max(4) / 4;

        if T::SIZE < 4 {
            self.stack[offset..offset + SLOT_SIZE].copy_from_slice(&[0u8; 4]);
        }
        self.stack[offset..offset + T::SIZE].copy_from_slice(value.to_le_bytes().as_ref());
    }

    pub fn pop<T: Number>(&mut self) -> T {
        self.idx -= T::SIZE.max(4) / 4;
        let offset = self.idx * SLOT_SIZE;
        T::from_le_bytes(&self.stack[offset..offset + T::SIZE])
    }

    pub fn drop<T: Number>(&mut self) {
        self.pop::<T>();
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

    pub fn cmp<T: Number>(&mut self) {
        let lhs = self.pop::<T>();
        let rhs = self.pop::<T>();
        self.push(rhs.cmp(&lhs) as i32);
    }

    pub fn dup<T: Number>(&mut self) {
        let idx = self.idx - T::SIZE.max(4) / 4;
        let offset = idx * SLOT_SIZE;
        let value = T::from_le_bytes(&self.stack[offset..offset + T::SIZE]);
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
