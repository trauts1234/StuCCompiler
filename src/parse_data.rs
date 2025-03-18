use std::collections::HashSet;

use crate::{enum_definition::EnumList, function_declaration::FunctionDeclaration};

#[derive(Clone, Debug)]
pub struct ParseData {
    variables: HashSet<String>,
    pub(crate) enums: EnumList,
    function_decls: Vec<FunctionDeclaration>,
}

impl ParseData {
    pub fn make_empty() -> ParseData {
        ParseData { variables: HashSet::new(), enums: EnumList::new(), function_decls: Vec::new() }
    }

    pub fn func_declarations_as_slice(&self) -> &[FunctionDeclaration] {
        self.function_decls.as_slice()
    }
    
    pub fn add_declaration(&mut self, toadd: FunctionDeclaration) {
        if self.get_function_declaration(&toadd.function_name).is_some() {
            return;//already declared, skip it
        }

        self.function_decls.push(toadd);
    }

    pub fn get_function_declaration(&self, func_name: &str) -> Option<&FunctionDeclaration> {
        self.function_decls.iter()
        .find(|func| func.function_name == func_name)
    }

    pub fn add_variable(&mut self, name: &str) {
        if self.variables.contains(name) {
            panic!("double definition of variable: {}",name);
        }

        self.variables.insert(name.to_string());
    }
    pub fn variable_defined(&mut self, name: &str) -> bool {
        self.variables.contains(name)
    }
}