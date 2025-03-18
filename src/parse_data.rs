use std::collections::HashSet;

use crate::{data_type::data_type::DataType, enum_definition::EnumList, function_declaration::FunctionDeclaration};

#[derive(Debug)]
pub struct ParseData {
    variables: HashSet<String>,
    pub(crate) enums: EnumList,
    function_decls: Vec<FunctionDeclaration>,

    local_symbol_table: Vec<(String, DataType)>//this is filled slowly, so do not read from it
}

impl ParseData {
    pub fn make_empty() -> ParseData {
        ParseData { variables: HashSet::new(), enums: EnumList::new(), function_decls: Vec::new(), local_symbol_table: Vec::new() }
    }

    /**
     * clones myself in a way that the returned parsedata is suitable for being used in a nested scope
     */
    pub fn clone_for_new_scope(&self) -> ParseData {
        ParseData { 
            variables: self.variables.clone(),
            enums: self.enums.clone(),
            function_decls: self.function_decls.clone(),
            local_symbol_table: Vec::new()//as in new scope, all symbols are in an outer scope
        }
    }

    pub fn func_declarations_as_vec(&self) -> Vec<FunctionDeclaration> {
        self.function_decls.clone()
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

    pub fn add_variable(&mut self, name: &str, data_type: DataType) {
        self.variables.insert(name.to_string());

        if self.local_symbol_table.iter().any(|(x,_)| x == name) {
            panic!("redefinition of variable {} in local scope", name);
        }

        self.local_symbol_table.push((name.to_string(), data_type));
    }

    /**
     * if a struct is used, then run this function, so that the struct can be forward declared
     */
    pub fn notify_struct_declaration(&mut self, name: &str) {
        self.variables.insert(name.to_string());
    }
    pub fn variable_defined(&mut self, name: &str) -> bool {
        self.variables.contains(name)
    }

    pub fn get_symbol_table(&self) -> &Vec<(String, DataType)> {
        &self.local_symbol_table
    }
}