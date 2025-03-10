use memory_size::MemoryLayout;

use crate::{data_type::data_type::DataType, declaration::{AddressedDeclaration, Declaration}, memory_size};


/**
 * a list of all the variables found so far
 */
#[derive(Clone)]
pub struct StackVariables {
    vars: Vec<(Declaration, MemoryLayout)>,//the variable, and offset from bp
    stack_used: MemoryLayout,

    outer_function_return_type: DataType
}

impl StackVariables {
    pub fn new_in_func_body(func_args:Vec<Declaration>, func_return_value: &DataType) -> StackVariables {
        let mut stack_vars = StackVariables {
            vars: Vec::new(),
            stack_used: MemoryLayout::new(),
            outer_function_return_type: func_return_value.clone(),
        };

        stack_vars.add_variables(func_args);

        stack_vars
    }

    pub fn get_stack_used(&self) -> MemoryLayout {
        self.stack_used
    }

    /**
     * will get the variable OR arg that has this name
     */
    pub fn get_variable(&self, var_name: &str) -> Option<AddressedDeclaration> {
        for (var, location) in &self.vars {
            if var.get_name() == var_name {
                return Some(AddressedDeclaration {
                    decl: var.clone(),
                    stack_offset: location.clone()
                });
            }
        }
        None
    }

    pub fn add_variable(&mut self, decl: Declaration) {
        assert!(self.get_variable(decl.get_name()).is_none());//variable should not already exist
        self.stack_used += decl.get_type().memory_size();//decrement the stack pointer first
        self.vars.push((decl, self.stack_used));//then add the variable as I don't want to overwrite the return value
    }
    pub fn add_variables(&mut self, decls: Vec<Declaration>) {
        for decl in decls {
            self.add_variable(decl);
        }
    }
}