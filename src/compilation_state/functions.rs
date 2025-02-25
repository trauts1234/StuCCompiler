use crate::{function_declaration::FunctionDeclaration, function_definition::FunctionDefinition};



#[derive(Debug)]
pub struct FunctionList {
    func_definitions: Vec<FunctionDefinition>,//all function definitions made in this translation unit
    all_func_declarations: Vec<FunctionDeclaration>,//ALL declarations, including declarations made from func definitions and straight up declarations
}

impl FunctionList {
    pub fn new() -> FunctionList {
        FunctionList{
            func_definitions: Vec::new(),
            all_func_declarations: Vec::new()
        }
    }

    pub fn func_definitions_as_slice(&self) -> &[FunctionDefinition] {
        &self.func_definitions
    }
    pub fn func_declarations_as_slice(&self) -> &[FunctionDeclaration] {
        &self.all_func_declarations
    }
    
    pub fn add_function(&mut self, toadd: FunctionDefinition) {
        assert!(self.get_function_definition(toadd.get_name()).is_none());//function can't already be defined
        self.all_func_declarations.push(toadd.as_decl());
        self.func_definitions.push(toadd);
    }
    pub fn add_declaration(&mut self, toadd: FunctionDeclaration) {
        if self.get_function_declaration(&toadd.function_name).is_some() {
            return;//already declared, skip it
        }

        self.all_func_declarations.push(toadd);
    }

    pub fn get_function_declaration(&self, func_name: &str) -> Option<&FunctionDeclaration> {
        self.all_func_declarations.iter()
        .find(|func| func.function_name == func_name)
    }
    pub fn get_function_definition(&self, func_name: &str) -> Option<&FunctionDefinition> {
        self.func_definitions.iter()
        .find(|func| func.get_name() == func_name)
    }
}