use crate::function_definition::FunctionDefinition;



#[derive(Debug)]
pub struct FunctionList {
    funcs: Vec<FunctionDefinition>
}

impl FunctionList {
    pub fn new() -> FunctionList {
        FunctionList{
            funcs: Vec::new()
        }
    }

    pub fn funcs_as_slice(&self) -> &[FunctionDefinition] {
        &self.funcs
    }
    pub fn add_function(&mut self, toadd: FunctionDefinition) {
        self.funcs.push(toadd);
    }

    pub fn get_function(&self, func_name: &str) -> Option<&FunctionDefinition> {
        self.funcs.iter()
        .find(|func| func.get_name() == func_name)
    }
}