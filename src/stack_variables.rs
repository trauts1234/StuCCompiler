use memory_size::MemoryLayout;

use crate::{declaration::Declaration, memory_size};


#[derive(Clone)]
pub struct StackVariables {
    vars: Vec<Declaration>,
    stack_used: MemoryLayout
}

impl StackVariables {
    pub fn new() -> StackVariables {
        StackVariables{
            vars: Vec::new(),
            stack_used: MemoryLayout::new()
        }
    }

    pub fn get_stack_used(&self) -> MemoryLayout {
        self.stack_used
    }
    pub fn get_variables(&self) -> &Vec<Declaration> {
        &self.vars
    }

    pub fn add_variable(&mut self, decl: Declaration) {
        self.stack_used += decl.get_memory_usage();
        self.vars.push(decl);
    }
}