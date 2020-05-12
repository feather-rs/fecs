use alloc::collections::VecDeque;

/// Handles allocation of entity indices, for either the shared world
/// or some shard.
#[derive(Debug, Default)]
pub struct EntityIndexAllocator {
    /// The set of free indices. Once a world knows that an entity
    /// with some index cannot possibly be accessed, it can free that
    /// entity's index.
    free: VecDeque<u32>,
    /// The next entity index to allocate. Used if `free` is empty.
    next: u32,
}

impl EntityIndexAllocator {
    /// Creates a new, empty `EntityIndexAllocator`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocates a new entity index.
    pub fn alloc(&mut self) -> u32 {
        self.free.pop_front().unwrap_or_else(|| {
            self.next += 1;
            self.next - 1
        })
    }

    /// Frees an entity index.
    pub fn free(&mut self, index: u32) {
        self.free.push_back(index);
    }

    /// Returns the total number of allocated indices.
    pub fn allocated(&self) -> usize {
        self.next as usize - self.free.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocator() {
        let mut a = EntityIndexAllocator::new();

        let one = a.alloc();
        let two = a.alloc();
        let three = a.alloc();

        assert_eq!(a.allocated(), 3);
        assert_ne!(one, two);
        assert_ne!(two, three);

        a.free(two);
        assert_eq!(a.allocated(), 2);

        let four = a.alloc();
        assert_eq!(four, two);

        assert_eq!(a.allocated(), 3);
    }
}
