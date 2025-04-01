use crate::memory_size::MemoryLayout;

pub struct StackUsage {
    outer_scope_stack_used: MemoryLayout,
    variable_stack_used: MemoryLayout,
    scratch_stack_used: MemoryLayout,
}

impl StackUsage {
    pub fn new() -> Self {
        StackUsage { outer_scope_stack_used: MemoryLayout::new(), variable_stack_used: MemoryLayout::new(), scratch_stack_used: MemoryLayout::new() }
    }

    pub fn clone_for_new_scope(&self) -> Self {
        StackUsage { outer_scope_stack_used: self.get_stack_used(), variable_stack_used: MemoryLayout::new(), scratch_stack_used: MemoryLayout::new() }
    }

    /**
     * consumes self to prevent adding variables after using the stack size?
     * returns how much extra stack this scope uses
     */
    pub fn get_stack_used(&self) -> MemoryLayout {
        self.variable_stack_used + self.scratch_stack_used
    }

    /**
     * returns: the offset from rbp to store the variable
     */
    pub fn allocate_variable_stack(&mut self, variable_size: MemoryLayout) -> MemoryLayout {
        self.variable_stack_used += variable_size;

        self.outer_scope_stack_used + self.variable_stack_used
    }

    /**
     * returns: the offset from rbp to store the temporary data
     */
    pub fn allocate_scratch_stack(&mut self, scratch_required: MemoryLayout) -> MemoryLayout {
        self.scratch_stack_used += scratch_required;

        self.outer_scope_stack_used + self.variable_stack_used + self.scratch_stack_used//scratch address is after variable storage
    }
}