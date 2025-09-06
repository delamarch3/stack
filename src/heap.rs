use std::ptr;
use std::sync::Mutex;

pub struct Allocation {
    free: bool,
    mem: Box<[u8]>,
}

impl Allocation {
    pub fn new(size: usize) -> Self {
        let free = false;
        let mem = vec![0; size].into_boxed_slice();

        Self { free, mem }
    }
}

#[derive(Default)]
pub struct Heap {
    allocations: Mutex<Vec<Allocation>>,
    free: Mutex<Vec<usize>>,
}

impl Heap {
    pub fn alloc(&self, size: usize) -> usize {
        let mut allocations = self.allocations.lock().unwrap();
        let mut free = self.free.lock().unwrap();

        let mut found = None;
        for (i, id) in free.iter().enumerate() {
            if let Some(a) = allocations.get(*id) {
                if a.mem.len() >= size {
                    found = Some((i, *id))
                }
            }
        }

        if let Some((i, id)) = found {
            allocations[id].free = false;
            free.remove(i);

            return id;
        }

        let a = Allocation::new(size);
        let id = allocations.len();
        allocations.push(a);

        id
    }

    pub fn ptr(&self, id: usize) -> *const u8 {
        let allocations = self.allocations.lock().unwrap();
        let Some(allocation) = allocations.get(id) else {
            return ptr::null();
        };

        allocation.mem.as_ptr()
    }

    pub fn free(&self, id: usize) {
        let mut allocations = self.allocations.lock().unwrap();
        let mut free = self.free.lock().unwrap();

        if let Some(allocation) = allocations.get_mut(id) {
            allocation.free = true;
            free.push(id);
        }
    }

    pub fn read(&self, id: usize, offset: usize, dst: &mut [u8]) -> bool {
        let mut allocations = self.allocations.lock().unwrap();

        let Some(allocation) = allocations.get_mut(id) else {
            return false;
        };

        let size = dst.len();
        let src = &allocation.mem[offset..];
        dst[..].copy_from_slice(&src[..size]);

        true
    }

    pub fn write(&self, id: usize, offset: usize, src: &[u8]) -> bool {
        let mut allocations = self.allocations.lock().unwrap();

        let Some(allocation) = allocations.get_mut(id) else {
            return false;
        };

        let dst = &mut allocation.mem[offset..];
        dst[..src.len()].copy_from_slice(src);

        true
    }
}
