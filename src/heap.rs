use std::sync::Mutex;

#[derive(Default)]
pub struct Heap {
    allocations: Mutex<Vec<Box<[u8]>>>,
}

impl Heap {
    pub fn alloc(&self, size: usize) -> usize {
        let mut allocations = self.allocations.lock().unwrap();

        let a = vec![0; size].into_boxed_slice();
        let id = allocations.len();
        allocations.push(a);

        id
    }

    pub fn free(&self, id: usize) {
        let mut allocations = self.allocations.lock().unwrap();

        // TODO: this won't work, the references of all of the allocations that follow it will
        // become invalid
        allocations.remove(id);
    }

    pub fn read(&self, id: usize, offset: usize, dst: &mut [u8]) -> bool {
        let mut allocations = self.allocations.lock().unwrap();

        let Some(allocation) = allocations.get_mut(id) else {
            return false;
        };

        let size = dst.len();
        let src = &allocation[offset..];
        dst[..].copy_from_slice(&src[..size]);

        true
    }

    pub fn write(&self, id: usize, offset: usize, src: &[u8]) -> bool {
        let mut allocations = self.allocations.lock().unwrap();

        let Some(allocation) = allocations.get_mut(id) else {
            return false;
        };

        let dst = &mut allocation[offset..];
        dst[..src.len()].copy_from_slice(src);

        true
    }
}
