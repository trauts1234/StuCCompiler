use memory_size::MemorySize;

use super::{ir_global_allocation_type::GlobalAllocationType, ir_stack_allocation::{VarIdentifier, VariableInfo}};



/// this struct stores where variables are stored, for all storage durations
/// this should be persistent across the entire file
#[derive(Debug, Default)]
pub struct IRMemoryManagement {
    /// This is used to generate labels for static variables
    current_function_name: String,
    
    /// this stores all variables that will be allocated on the stack.
    /// It does not have to be in order, since memory will not be allocated for these variables yet
    stack: Vec<VariableInfo>,

    /// this stores all extern, static and global variables, alongside what type of storage duration they have
    global_vars: Vec<(VariableInfo, GlobalAllocationType)>,

}

impl IRMemoryManagement {
    pub fn reset_for_new_function(&mut self, func_name: String) {
        self.current_function_name = func_name;//sets this function's name
        self.stack.clear();//purges stack variables from the previous function
    }

    pub fn find_variable(&self, var_id: &VarIdentifier) -> &VariableInfo {
        self.stack.iter()//start by searching the stack
        .rev()//find the closest defined variable first, to enforece variable shadowing
        .chain(//then try searching global/extern/static variables
            self.global_vars.iter()
            .map(|(x,_)| x)//extract the name only
        )
        .find(|x| x.identifier() == var_id)//search for the variable
        .unwrap()
    }

    pub fn add_stack_variable(&mut self, var: VariableInfo) {
        self.stack.push(var);
    }
}