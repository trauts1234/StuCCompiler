use std::collections::HashMap;


use crate::{compilation_state::label_generator::LabelGenerator, data_type::recursive_data_type::DataType, enum_definition::EnumList, function_declaration::FunctionDeclaration, struct_definition::{StructIdentifier, UnpaddedStructDefinition}};

#[derive(Debug)]
pub struct ParseData {
    pub(crate) enums: EnumList,
    typedefs: HashMap<String, DataType>,
    function_decls: Vec<FunctionDeclaration>,
    structs: Vec<(StructIdentifier, UnpaddedStructDefinition)>,//defined and declared structs

    local_symbol_table: Vec<(String, DataType)>,//this is filled slowly, so do not read from it
}

impl ParseData {
    pub fn make_empty() -> Self {
        Self {
            enums: EnumList::default(),
            typedefs: HashMap::new(),
            function_decls: Vec::new(),
            structs: Vec::new(),
            local_symbol_table: Vec::new(),
        }
    }

    /**
     * clones myself in a way that the returned parsedata is suitable for being used in a nested scope
     */
    pub fn clone_for_new_scope(&self) -> Self {
        Self {
            enums: self.enums.clone(),
            typedefs: self.typedefs.clone(),
            function_decls: self.function_decls.clone(),
            structs: self.structs.clone(),
            local_symbol_table: Vec::new(),
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

    /// saves the struct definition under the name specified
    /// returns an identifier for the struct added
    /// if the struct was previously *declared*, it is overwritten with new contents
    pub fn add_struct(&mut self, name: &Option<String>, new_definition: &UnpaddedStructDefinition, struct_label_generator: &mut LabelGenerator) -> StructIdentifier {

        let defined_struct_finder =
            self.structs
            .iter()
            .rev()//search from newest to oldest
            .find(|(ident,data)|
                //find a struct where the name is the same
                ident.name == *name &&
                //and has defined members
                data.ordered_members.is_some()
            )
            .cloned();

        let declared_struct_finder = 
            self.structs
            .iter_mut()
            .rev()//search from newest to oldest
            .find(|(ident,data)|
                //find a struct where the name is the same
                ident.name == *name &&
                //and does not have defined members (is previously declared)
                data.ordered_members.is_none()
            );
        
        match (declared_struct_finder, &defined_struct_finder, new_definition.ordered_members.is_some()) {
            (Some(decl), _, _) => {
                //found declared struct, overwrite data with new definition(could still be null btw)
                decl.1 = new_definition.clone();

                decl.0.clone()//return the declaration identifier
            },
            (None, _, true) | //defined an undeclared struct
            (None, None, false) // declared an undeclared struct
            => {
                let identifier = StructIdentifier {
                    name: name.clone(),
                    id: struct_label_generator.generate_label_number(),
                };
                self.structs.push((identifier.clone(), new_definition.clone()));//add new struct, overwriting if it was declared etc.
    
                identifier//return the new identifier
            },
            (None, Some(define), false) => {
                //I declared an already-defined struct

                define.0.clone()
            },
        }
    }

    pub fn get_all_structs(&self) -> &[(StructIdentifier, UnpaddedStructDefinition)] {
        &self.structs
    }

    pub fn add_typedef(&mut self, name: String, new_type: DataType) {
        //can be overwritten, insert new type
        self.typedefs.insert(name, new_type);
    }

    pub fn get_typedef(&self, name: &str) -> Option<&DataType> {
        self.typedefs.get(name)
    }
}