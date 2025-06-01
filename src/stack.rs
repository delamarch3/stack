const STACK_SIZE: usize = 64;
pub struct OperandStack {
    stack: [i32; STACK_SIZE],
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
        let until = if self.ptr < 64 - 8 {
            self.ptr + 8
        } else {
            self.ptr
        };

        let width = 8;
        let mut sep = "";
        write!(f, "[")?;
        for n in &self.stack[..until] {
            write!(f, "{sep}")?;
            write!(f, "{n:width$}")?;
            sep = ",";
        }
        write!(f, "]\n")?;

        let cursor = self.ptr + (self.ptr * width);
        write!(f, "{:cursor$}^", "")
    }
}

impl OperandStack {
    pub fn size(&self) -> usize {
        self.ptr
    }

    pub fn push(&mut self, value: i32) {
        self.stack[self.ptr] = value;
        self.ptr += 1;
    }

    pub fn pop(&mut self) -> i32 {
        self.ptr -= 1;
        self.stack[self.ptr]
    }

    pub fn add(&mut self) {
        let (a, b) = (self.pop(), self.pop());
        let value = a + b;
        self.push(value);
    }

    pub fn sub(&mut self) {
        let (a, b) = (self.pop(), self.pop());
        let value = b - a;
        self.push(value);
    }

    pub fn mul(&mut self) {
        let (a, b) = (self.pop(), self.pop());
        let value = a * b;
        self.push(value);
    }

    pub fn div(&mut self) {
        let (a, b) = (self.pop(), self.pop());
        let value = b / a;
        self.push(value);
    }

    pub fn cmp(&mut self, lhs: i32) {
        let rhs = self.pop();
        self.push(rhs.cmp(&lhs) as i32);
    }

    pub fn swap(&mut self) {
        let a = self.stack[self.ptr - 1];
        let b = self.stack[self.ptr - 2];
        self.stack[self.ptr - 2] = a;
        self.stack[self.ptr - 1] = b;
    }

    pub fn dup(&mut self) {
        self.push(self.stack[self.ptr - 1]);
    }

    pub fn over(&mut self) {
        self.push(self.stack[self.ptr - 2]);
    }

    pub fn rot(&mut self) {
        self.push(self.stack[self.ptr - 3]);
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
        stack.add();
        assert_eq!(stack.pop(), 25);
    }
}
