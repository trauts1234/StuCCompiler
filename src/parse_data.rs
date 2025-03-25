use std::collections::HashSet;

use indexmap::IndexMap;

use crate::{data_type::data_type::DataType, enum_definition::EnumList, function_declaration::FunctionDeclaration, struct_definition::UnpaddedStructDefinition};

#[derive(Debug)]
pub struct ParseData {
    variables: HashSet<String>,
    pub(crate) enums: EnumList,
    function_decls: Vec<FunctionDeclaration>,
    structs: IndexMap<String, UnpaddedStructDefinition>,//defined and declared structs

    local_symbol_table: IndexMap<String, DataType>,//this is filled slowly, so do not read from it
}

impl ParseData {
    pub fn make_empty() -> ParseData {
        ParseData { variables: HashSet::new(), enums: EnumList::new(), function_decls: Vec::new(),  local_symbol_table: IndexMap::new(), structs: IndexMap::new()}
    }

    /**
     * clones myself in a way that the returned parsedata is suitable for being used in a nested scope
     */
    pub fn clone_for_new_scope(&self) -> ParseData {
        ParseData { 
            variables: self.variables.clone(),
            enums: self.enums.clone(),
            function_decls: self.function_decls.clone(),
            structs: self.structs.clone(),

            local_symbol_table: IndexMap::new(), //as in new scope, all symbols are in an outer scope
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

        self.local_symbol_table.insert(name.to_string(), data_type);
    }
    pub fn variable_defined(&self, name: &str) -> bool {
        self.variables.contains(name)
    }

    pub fn get_symbol_table(&self) -> &IndexMap<String, DataType> {
        &self.local_symbol_table
    }

    pub fn add_struct(&mut self, new_definition: &UnpaddedStructDefinition) {
        let new_struct_name = new_definition.name.as_ref().unwrap();

        if let Some(definition) = self.structs.get_mut(new_struct_name) {
            match (&definition.ordered_members, &new_definition.ordered_members) {
                (Some(_), Some(_)) => panic!("redefinition of struct {}", definition.name.clone().unwrap()),

                (None, Some(_)) => definition.ordered_members = new_definition.ordered_members.clone(),//new definition contains more data

                _ => {}//new definition provides no new data
            }
        } else {
            self.structs.insert(new_struct_name.to_string(), new_definition.clone());//add new struct
        }
    }

    pub fn get_struct(&self, name: &str) -> Option<&UnpaddedStructDefinition> {
        self.structs.get(name)
    }

    pub fn get_all_structs(&self) -> &IndexMap<String, UnpaddedStructDefinition> {
        &self.structs
    }
}