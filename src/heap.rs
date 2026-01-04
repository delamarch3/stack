use std::sync::Mutex;

pub struct Allocation {
    mem: Box<[u8]>,
}

impl Allocation {
    pub fn new(size: usize) -> Self {
        let mem = vec![0; size].into_boxed_slice();

        Self { mem }
    }
}

#[derive(Default)]
pub struct Heap {
    allocations: Mutex<Vec<Allocation>>,
}

impl Heap {
    pub fn alloc(&self, size: usize) -> *const u8 {
        let mut allocations = self.allocations.lock().unwrap();

        let alloc = Allocation::new(size);
        let ptr = alloc.mem.as_ptr();
        allocations.push(alloc);

        ptr
    }

    pub fn free(&self, ptr: *const u8) {
        let mut allocations = self.allocations.lock().unwrap();

        let Some((i, _)) = allocations
            .iter()
            .enumerate()
            .find(|(_, alloc)| alloc.mem.as_ptr() == ptr)
        else {
            return;
        };

        allocations.swap_remove(i);
    }
}
