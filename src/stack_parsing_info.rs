use memory_size::MemoryLayout;

/**
 * this struct stores information about the current state of the stack, allowing the parser to allocate memory
 */
pub struct StackInfo {
    per_scope_size: Vec<MemoryLayout>,//for each scope opened, an element in vec with the current memory size
    diff_from_bp: MemoryLayout//the total distance from rbp in (sum of curr_qwords)
}

impl StackInfo {
    pub fn new() -> StackInfo {
        StackInfo {
            per_scope_size: vec![MemoryLayout::new()],//starts with one scope, and no elements allocated
            diff_from_bp: MemoryLayout::new()
        }
    }

    /**
     * allocates 64 bits for you to use on the stack. only valid for the current scope
     * returns a memory layout saying how many bytes to subtract from the base pointer
     */
    pub fn allocate_qword(&mut self) -> MemoryLayout {
        let allocation_size = MemoryLayout::from_bytes(8);
        //grow the stack downwards by the allocation size
        self.per_scope_size.last_mut().unwrap().increment_by(&allocation_size);
        self.diff_from_bp.increment_by(&allocation_size);

        //note: subtract from the base pointer first, then return the new pointer, as otherwise we might overwrite the data that RBP is pointing to
        self.diff_from_bp.clone()
    }
    
    /**
     * creates a new segment, to store variables that only appear in this stack frame
     */
    pub fn enter_scope(&mut self) {
        self.per_scope_size.push(MemoryLayout::new());
    }
    /**
     * removes the segment as we have left the scope and can overwrite the local variables
     */
    pub fn exit_scope(&mut self) {
        self.per_scope_size.pop();
    }
}