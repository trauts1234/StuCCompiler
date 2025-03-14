use memory_size::MemoryLayout;

use crate::{data_type::{base_type::BaseType, data_type::DataType}, declaration::{AddressedDeclaration, Declaration, VariableAddress}, memory_size};


/**
 * a list of all the variables found so far
 */
#[derive(Clone, Debug)]
pub struct StackVariables {
    vars: Vec<AddressedDeclaration>,//the variable, and offset from bp
    stack_used: MemoryLayout,

    outer_function_return_type: DataType
}

impl StackVariables {
    pub fn new() -> StackVariables {
        StackVariables { vars: Vec::new(), stack_used: MemoryLayout::new(), outer_function_return_type: DataType::new_from_base_type(&BaseType::VOID, &Vec::new()) }
    }

    pub fn set_return_type(&mut self, ret_type: &DataType) {
        self.outer_function_return_type = ret_type.clone();
    }
    pub fn get_return_type(&self) -> &DataType {
        &self.outer_function_return_type
    }

    /**
     * will get the variable OR arg that has this name
     */
    pub fn get_variable(&self, var_name: &str) -> Option<&AddressedDeclaration> {
        for decl in &self.vars {
            if decl.decl.get_name() == var_name {
                return Some(decl);
            }
        }
        None
    }

    pub fn add_stack_variable(&mut self, decl: Declaration) {
        assert!(self.get_variable(decl.get_name()).is_none());//variable should not already exist
        self.stack_used += decl.get_type().memory_size();//decrement the stack pointer first
        self.vars.push(AddressedDeclaration { decl, location: VariableAddress::STACKOFFSET(self.stack_used) });//then add the variable as I don't want to overwrite the return value
    }
    pub fn add_global_variable(&mut self, decl: Declaration) {
        assert!(self.get_variable(decl.get_name()).is_none());//variable should not already exist
        self.vars.push(AddressedDeclaration{decl, location:VariableAddress::CONSTANTADDRESS});
    }
}