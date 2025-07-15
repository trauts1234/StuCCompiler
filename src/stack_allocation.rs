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

    /// Allocates with a requirement of alignment of the returned offset
    /// 
    /// Returns the number of bytes to subtract from RBP to access the memory
    pub fn allocate_aligned(&mut self, size: MemorySize, alignment: MemorySize) -> MemorySize {
        self.allocated += aligned_size(size, alignment);
        assert!(self.allocated.size_bytes() % alignment.size_bytes() == 0);
        self.allocated
    }

    /// consume self and return the stack required
    pub fn stack_required(self) -> MemorySize {
        self.allocated
    }

    /// When taking if/else branches, generate copies of myself, to be merged later
    pub fn split_for_branching(&self) -> (Self, Self) {
        (
            Self {allocated:self.allocated},
            Self {allocated:self.allocated}
        )
    }
    pub fn merge_from_branching(&mut self, branch_l: Self, branch_r: Self) {
        assert!(branch_l.allocated >= self.allocated);
        assert!(branch_r.allocated >= self.allocated);

        self.allocated = MemorySize::max(
            branch_l.allocated,
            branch_r.allocated,
        )
    }
}

/**
 * calculates how much extra memory is needed to make current_offset a multiple of alignment
 */
pub fn align(current_offset: MemorySize, alignment: MemorySize) -> MemorySize {
    let bytes_past_last_boundary = current_offset.size_bytes() % alignment.size_bytes();

    MemorySize::from_bytes (
        (alignment.size_bytes() - bytes_past_last_boundary) % alignment.size_bytes()
    )
}

/**
 * calculates the size of current_offset when rounded up to the alignment boundary
 * return value >= current_offset
 */
pub fn aligned_size(current_offset: MemorySize, alignment: MemorySize) -> MemorySize {
    current_offset + align(current_offset, alignment)
}