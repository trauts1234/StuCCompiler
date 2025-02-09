use memory_size::MemoryLayout;

use crate::{declaration::Declaration, memory_size};


#[derive(Clone)]
pub struct StackVariables {
    vars: Vec<(Declaration, MemoryLayout)>,//the variable, and offset from bp
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
    pub fn get_variable_bp_offset(&self, var_name: &str) -> Option<MemoryLayout> {
        for (var, location) in &self.vars {
            if var.get_name() == var_name {
                return Some(*location);
            }
        }
        None
    }

    pub fn add_variable(&mut self, decl: Declaration) {
        self.stack_used += decl.get_memory_usage();//decrement the stack pointer first
        self.vars.push((decl, self.stack_used));//then add the variable as I don't want to overwrite the return value
    }
    pub fn add_variables(&mut self, decls: Vec<Declaration>) {
        for decl in decls {
            self.add_variable(decl);
        }
    }
}