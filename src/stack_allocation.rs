use memory_size::MemorySize;

#[derive(Default)]
/// Keeps track of alloctions on the stack, and returns
pub struct StackAllocator {
    allocated: MemorySize
}

impl StackAllocator {
    pub fn with_previous_allocations(stack_already_used: MemorySize) -> Self {
        Self { allocated: stack_already_used }
    }
    /// Allocates with no alignment on the stack
    /// 
    /// Returns the number of bytes to subtract from RBP to access the memory
    pub fn allocate(&mut self, size: MemorySize) -> MemorySize {
        self.allocated += size;
        self.allocated
    }

    pub fn allocate_aligned(&mut self, size: MemorySize, alignment: MemorySize) -> MemorySize {
        todo!()
    }
}