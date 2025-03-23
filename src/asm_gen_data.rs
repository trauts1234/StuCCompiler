use crate::{data_type::data_type::DataType, function_declaration::FunctionDeclaration, memory_size::MemoryLayout, parse_data::ParseData, struct_definition::{StructDefinition, StructList}};
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
    pub(crate) data_type: DataType,
    pub(crate) location: VariableAddress
}

#[derive(Clone, Debug)]
pub struct AsmData {
    variables: IndexMap<String, AddressedDeclaration>,//hashmap, but keeps order to keep the stack sorted correctly
    function_decls: Vec<FunctionDeclaration>,
    current_function_return_type: DataType,
    current_stack_size: MemoryLayout,//difference of RSP and RBP, positive number
    struct_list: StructList,
}

impl AsmData {
    pub fn new_for_global_scope(parse_data: &ParseData) -> AsmData {
        let global_variables = parse_data.get_symbol_table()
            .iter()
            .map(add_global_variable)
            .collect();

        AsmData {
            variables: global_variables,//store global variables
            function_decls: parse_data.func_declarations_as_vec(),//store possible functions to call
            current_function_return_type: DataType::make_void(),//global namespace has no return type
            current_stack_size: MemoryLayout::new(),//no stack currently used
            struct_list:parse_data.structs.clone(),//use structs declared in the global scope
        }
    }
    pub fn clone_for_new_scope(&self, parse_data: &ParseData, current_function_return_type: DataType) -> AsmData {
        let mut new_stack_height = self.current_stack_size;

        let local_variables = parse_data.get_symbol_table()
            .iter()
            .map(|(var_name, var_type)| add_variable(&mut new_stack_height, var_name, var_type));//add each variable and generate metadata

        //add all current variables then overwrite with local variables (shadowing)
        let variables: IndexMap<String, AddressedDeclaration> = self.variables.clone().into_iter().chain(local_variables).collect();
        //same for structs
        let structs = self.struct_list.merge(&parse_data.structs);


        AsmData { 
            variables,
            function_decls: parse_data.func_declarations_as_vec(),
            current_function_return_type,
            current_stack_size: new_stack_height,
            struct_list:structs
        }
    }

    pub fn get_stack_height(&self) -> MemoryLayout {
        self.current_stack_size
    }

    pub fn get_function_declaration(&self, func_name: &str) -> Option<&FunctionDeclaration> {
        self.function_decls.iter()
        .find(|func| func.function_name == func_name)
    }

    pub fn get_variable(&self, name: &str) -> &AddressedDeclaration {
        self.variables.get(name).unwrap()
    }
    pub fn get_function_return_type(&self) -> &DataType {
        &self.current_function_return_type
    }
    pub fn get_struct(&self, name: &str) -> &StructDefinition {
        self.struct_list.get_struct(name).unwrap()
    }
}

fn add_variable(stack_height: &mut MemoryLayout, var_name: &str, var_type: &DataType) -> (String, AddressedDeclaration) {

    *stack_height += var_type.memory_size();//increase stack pointer to store extra variable

    let decl = AddressedDeclaration { data_type: var_type.clone(), location: VariableAddress::STACKOFFSET(stack_height.clone()) };//then generate address, as to not overwrite the stack frame

    (var_name.to_string(), decl)
}

/**
 * note this takes a tuple, so that it can be run in an iterator map()
 */
fn add_global_variable(data: (&String, &DataType)) -> (String, AddressedDeclaration) {
    let (var_name, var_type) = data;
    (var_name.to_string(), AddressedDeclaration{ data_type: var_type.clone(), location: VariableAddress::CONSTANTADDRESS })
}