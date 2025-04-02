use crate::{data_type::{base_type::BaseType, recursive_data_type::RecursiveDataType}, function_declaration::FunctionDeclaration, memory_size::MemoryLayout, parse_data::ParseData, struct_definition::StructDefinition};
use indexmap::IndexMap;

/**
 * represents an addressing mode for variables
 * offset from stack or
 * constant memory location(global variable)
 */
#[derive(Debug, Clone)]
pub enum VariableAddress{
    STACKOFFSET(MemoryLayout),//number of bytes below RBP
    CONSTANTADDRESS
}

#[derive(Debug, Clone)]
pub struct AddressedDeclaration {
    pub(crate) data_type: RecursiveDataType,
    pub(crate) location: VariableAddress
}

#[derive(Clone, Debug)]
pub struct AsmData {
    variables: IndexMap<String, AddressedDeclaration>,//hashmap, but keeps order to keep the stack sorted correctly
    function_decls: Vec<FunctionDeclaration>,
    current_function_return_type: RecursiveDataType,
    struct_list: IndexMap<String, StructDefinition>,//needs to be ordered since some structs need previously declared structs as members
}

impl AsmData {
    pub fn new_for_global_scope(parse_data: &ParseData) -> AsmData {
        let global_variables = parse_data.get_symbol_table()
            .iter()
            .map(generate_global_variable_decl)
            .collect();

        let mut result = AsmData {
            variables: global_variables,//store global variables
            function_decls: parse_data.func_declarations_as_vec(),//store possible functions to call
            current_function_return_type: RecursiveDataType::RAW(BaseType::VOID),//global namespace has no return type
            struct_list:IndexMap::new()//will get filled soon
        };

        for (name, unpadded) in parse_data.get_all_structs().iter() {
            result.struct_list.insert(name.to_string(), unpadded.pad_members(&result));//add structs in order
        }

        result
    }
    pub fn clone_for_new_scope(&self, parse_data: &ParseData, current_function_return_type: RecursiveDataType, stack_data: &mut MemoryLayout) -> AsmData {
        let mut result = self.clone();

        //add functions
        result.function_decls = parse_data.func_declarations_as_vec();

        //set return type
        result.current_function_return_type = current_function_return_type;

        //add new structs
        for (name, unpadded) in parse_data.get_all_structs().iter() {
            result.struct_list.insert(name.to_string(), unpadded.pad_members(&result));//add new structs in order
        }

        //when creating local variables, I need struct data beforehand
        let local_variables: Vec<_> = parse_data.get_symbol_table().iter().map(|(a,b)| (a.clone(), b.clone())).collect();

        //overwrite stack variable symbols with local variables (shadowing)
        for (name, var_type) in local_variables {
            let var_size = var_type.memory_size(&result);

            *stack_data += var_size;
            let var_address_offset = stack_data.clone();//increase stack pointer to store extra variable

            let decl = AddressedDeclaration { data_type: var_type.clone(), location: VariableAddress::STACKOFFSET(var_address_offset.clone()) };//then generate address, as to not overwrite the stack frame

            result.variables.shift_remove(&name);//ensure the new variable is put on the front of the indexmap
            result.variables.insert(name, decl);
        }

        result
    }

    pub fn get_function_declaration(&self, func_name: &str) -> Option<&FunctionDeclaration> {
        self.function_decls.iter()
        .find(|func| func.function_name == func_name)
    }

    pub fn get_variable(&self, name: &str) -> &AddressedDeclaration {
        self.variables.get(name).unwrap()
    }
    pub fn get_function_return_type(&self) -> &RecursiveDataType {
        &self.current_function_return_type
    }
    pub fn get_struct(&self, name: &str) -> &StructDefinition {
        self.struct_list.get(name).unwrap()
    }
}


/**
 * note this takes a tuple, so that it can be run in an iterator map()
 */
fn generate_global_variable_decl(data: (&String, &RecursiveDataType)) -> (String, AddressedDeclaration) {
    let (var_name, var_type) = data;
    (var_name.to_string(), AddressedDeclaration{ data_type: var_type.clone(), location: VariableAddress::CONSTANTADDRESS })
}