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
    pub fn alloc(&self, size: usize) -> *const u8 {
        let mut allocations = self.allocations.lock().unwrap();
        let mut free = self.free.lock().unwrap();

        let mut found = None;
        for (i, id) in free.iter().enumerate() {
            if let Some(alloc) = allocations.get(*id) {
                if alloc.mem.len() >= size {
                    found = Some((i, *id, alloc.mem.as_ptr()));
                    break;
                }
            }
        }

        if let Some((i, id, ptr)) = found {
            allocations[id].free = false;
            free.remove(i);

            return ptr;
        }

        let alloc = Allocation::new(size);
        let ptr = alloc.mem.as_ptr();
        allocations.push(alloc);

        ptr
    }

    pub fn free(&self, ptr: *const u8) {
        let mut allocations = self.allocations.lock().unwrap();
        let mut free = self.free.lock().unwrap();

        let Some((id, allocation)) = allocations
            .iter_mut()
            .enumerate()
            .find(|(_, alloc)| alloc.mem.as_ptr() == ptr)
        else {
            todo!()
        };

        allocation.free = true;
        free.push(id);
    }

    pub fn read(&self, ptr: *const u8, offset: usize, dst: &mut [u8]) -> bool {
        let allocations = self.allocations.lock().unwrap();

        let Some(allocation) = allocations.iter().find(|alloc| alloc.mem.as_ptr() == ptr) else {
            return false;
        };

        let size = dst.len();
        let src = &allocation.mem[offset..];
        dst[..].copy_from_slice(&src[..size]);

        true
    }

    pub fn write(&self, ptr: *const u8, offset: usize, src: &[u8]) -> bool {
        let mut allocations = self.allocations.lock().unwrap();

        let Some(allocation) = allocations
            .iter_mut()
            .find(|alloc| alloc.mem.as_ptr() == ptr)
        else {
            return false;
        };

        let dst = &mut allocation.mem[offset..];
        dst[..src.len()].copy_from_slice(src);

        true
    }
}
