pub struct Stack {
    stack: [i64; 64],
    ptr: usize,
}

impl Stack {
    pub fn new() -> Self {
        let stack = [0; 64];
        let ptr = 0;
        Self { stack, ptr }
    }

    pub fn push(&mut self, value: i64) {
        self.stack[self.ptr] = value;
        self.ptr += 1;
    }

    pub fn pop(&mut self) -> i64 {
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
        let value = a - b;
        self.push(value);
    }

    pub fn mul(&mut self) {
        let (a, b) = (self.pop(), self.pop());
        let value = a * b;
        self.push(value);
    }

    pub fn div(&mut self) {
        let (a, b) = (self.pop(), self.pop());
        let value = a / b;
        self.push(value);
    }
}

#[cfg(test)]
mod test {
    use super::Stack;

    #[test]
    fn test_stack() {
        let mut stack = Stack::new();
        stack.push(10);
        stack.push(15);
        stack.add();
        assert_eq!(stack.pop(), 25);
    }
}
