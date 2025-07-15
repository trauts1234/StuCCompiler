use memory_size::MemorySize;

#[derive(Default)]
/// Keeps track of alloctions on the stack, and returns
pub struct StackAllocator {
    allocated: MemorySize
}

impl StackAllocator {
    pub fn with_previous_allocations(stack_already_used: MemorySize) -> Self {
        Self { allocated: stack_already_used };
        panic!()
    }
    /// Allocates with no alignment on the stack
    /// 
    /// Returns the number of bytes to subtract from RBP to access the memory
    pub fn allocate(curr: &mut MemorySize, size: MemorySize) -> MemorySize {
        *curr += size;
        *curr
    }

    pub fn allocate_aligned(curr: &mut MemorySize, size: MemorySize, alignment: MemorySize) -> MemorySize {
        todo!()
    }

    /// For old functions that handle the stack manually, just give it a ref
    pub fn stack_data_mut(&mut self) -> &mut MemorySize {
        &mut self.allocated
    }
}