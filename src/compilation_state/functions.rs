use crate::{function_definition::FunctionDefinition, parse_data::ParseData};

pub struct FunctionList {
    func_definitions: Vec<FunctionDefinition>,//all function definitions made in this translation unit
}

impl FunctionList {
    /**
     * creates a function list that is empty
     */
    pub fn new() -> FunctionList {
        FunctionList{
            func_definitions: Vec::new(),
        }
    }


    pub fn func_definitions_as_slice(&self) -> &[FunctionDefinition] {
        &self.func_definitions
    }
    pub fn add_function(&mut self,scope_data: &mut ParseData, toadd: FunctionDefinition) {
        assert!(self.get_function_definition(toadd.get_name()).is_none());//function can't already be defined
        scope_data.add_declaration(toadd.as_decl());
        self.func_definitions.push(toadd);
    }
    pub fn get_function_definition(&self, func_name: &str) -> Option<&FunctionDefinition> {
        self.func_definitions.iter()
        .find(|func| func.get_name() == func_name)
    }
}