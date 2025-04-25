use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;

use crate::{data_type::recursive_data_type::DataType, enum_definition::EnumList, function_declaration::FunctionDeclaration, struct_definition::{StructIdentifier, UnpaddedStructDefinition}};

#[derive(Debug, Default, Clone)]
pub struct ParseData {
    pub(crate) enums: EnumList,
    typedefs: HashMap<String, DataType>,
    function_decls: Vec<FunctionDeclaration>,
    structs: IndexMap<StructIdentifier, UnpaddedStructDefinition>,//defined and declared structs

    local_symbol_table: Vec<(String, DataType)>,//this is filled slowly, so do not read from it
    next_free_struct_id: i32//anonymous structs get given an id from this
}

impl ParseData {
    pub fn make_empty() -> ParseData {
        ParseData::default()
    }

    /**
     * clones myself in a way that the returned parsedata is suitable for being used in a nested scope
     */
    pub fn clone_for_new_scope(&self) -> ParseData {
        let mut result = self.clone();
        result.local_symbol_table = Vec::new();//as in new scope, all symbols are in an outer scope

        result
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
        .rev()//search closest first
        .find(|func| func.function_name == func_name)
    }

    pub fn add_variable(&mut self, name: &str, data_type: DataType) {
        if self.local_symbol_table.iter().any(|(x,_)| x == name) {
            panic!("redefinition of variable {} in local scope", name);
        }

        self.local_symbol_table.push((name.to_string(), data_type));
    }

    pub fn get_symbol_table(&self) -> &Vec<(String, DataType)> {
        &self.local_symbol_table
    }

    pub fn add_struct(&mut self, name: &StructIdentifier, new_definition: &UnpaddedStructDefinition) {

        if let Some(definition) = self.structs.get_mut(name) {
            match (&definition.ordered_members, &new_definition.ordered_members) {
                (Some(_), Some(_)) => panic!("redefinition of struct {:?}", name),

                (None, Some(_)) => definition.ordered_members = new_definition.ordered_members.clone(),//new definition contains more data

                _ => {}//new definition provides no new data
            }
        } else {
            self.structs.insert(name.clone(), new_definition.clone());//add new struct
        }
    }

    pub fn get_struct(&self, name: &StructIdentifier) -> Option<&UnpaddedStructDefinition> {
        self.structs.get(name)
    }

    pub fn get_all_structs(&self) -> &IndexMap<StructIdentifier, UnpaddedStructDefinition> {
        &self.structs
    }

    pub fn add_typedef(&mut self, name: String, new_type: DataType) {
        //can be overwritten, insert new type
        self.typedefs.insert(name, new_type);
    }

    pub fn get_typedef(&self, name: &str) -> Option<&DataType> {
        self.typedefs.get(name)
    }

    pub fn generate_struct_id(&mut self) -> i32 {
        let result = self.next_free_struct_id;

        self.next_free_struct_id.checked_add(1).unwrap();

        result
    }
}