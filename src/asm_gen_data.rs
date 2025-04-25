use crate::{assembly::operand::{memory_operand::MemoryOperand, Operand}, compilation_state::label_generator::LabelGenerator, data_type::{base_type::BaseType, recursive_data_type::DataType}, function_declaration::FunctionDeclaration, parse_data::ParseData, struct_definition::{StructDefinition, StructIdentifier}};
use memory_size::MemorySize;

#[derive(Clone)]
pub struct AddressedDeclaration {
    pub(crate) data_type: DataType,
    pub(crate) location: Operand
}

#[derive(Clone)]
pub struct AsmData {
    variables: Vec<(String, AddressedDeclaration)>,
    current_function_return_type: DataType,
    struct_list: Vec<(StructIdentifier, StructDefinition)>,//needs to be ordered since some structs need previously declared structs as members
    break_label: Option<String>,//which label to jump to on a "break;" statement
}

/// Stores information that is required globally and does not change when entering new scopes, like the list of accessible functions
pub struct GlobalAsmData {
    function_decls: Vec<FunctionDeclaration>,
    label_gen: LabelGenerator
}

impl GlobalAsmData {
    pub fn new(global_parse_data: &ParseData) -> Self {
        Self {
            function_decls: global_parse_data.func_declarations_as_vec(),
            label_gen: LabelGenerator::default()
        }
    }

    pub fn get_function_declaration(&self, func_name: &str) -> Option<&FunctionDeclaration> {
        self.function_decls.iter()
        .find(|func| func.function_name == func_name)
    }

    pub fn label_gen_mut(&mut self) -> &mut LabelGenerator {
        &mut self.label_gen
    }
}

impl AsmData {
    pub fn new_for_global_scope(parse_data: &ParseData) -> AsmData {
        let global_variables = parse_data.get_symbol_table()
            .iter()
            .map(generate_global_variable_decl)
            .collect();

        let mut result = AsmData {
            variables: global_variables,//store global variables
            current_function_return_type: DataType::RAW(BaseType::VOID),//global namespace has no return type
            struct_list: Vec::new(),//will get filled soon
            break_label: None,//break cannot be called here
        };

        for (name, unpadded) in parse_data.get_all_structs().iter() {
            result.struct_list.push((name.clone(), unpadded.pad_members(&result)));//add structs in order
        }

        result
    }
    pub fn clone_for_new_function(&self, parse_data: &ParseData, current_function_return_type: DataType, stack_data: &mut MemorySize) -> AsmData {
        let mut result = self.clone();

        //set return type
        result.current_function_return_type = current_function_return_type;

        //add new structs
        for (name, unpadded) in parse_data.get_all_structs().iter() {
            result.struct_list.push((name.clone(), unpadded.pad_members(&result)));//add new structs in order
        }

        //when creating local variables, I need struct data beforehand
        let local_variables: Vec<_> = parse_data.get_symbol_table().iter().map(|(a,b)| (a.clone(), b.clone())).collect();

        //overwrite stack variable symbols with local variables (shadowing)
        for (name, var_type) in local_variables {
            let var_size = var_type.memory_size(&result);

            *stack_data += var_size;
            let var_address_offset = stack_data.clone();//increase stack pointer to store extra variable

            let decl = AddressedDeclaration { data_type: var_type.clone(), location: Operand::Mem(MemoryOperand::SubFromBP(var_address_offset.clone())) };//then generate address, as to not overwrite the stack frame

            result.variables.push((name, decl));
        }

        result
    }

    pub fn clone_for_new_scope(&self, parse_data: &ParseData, stack_data: &mut MemorySize) -> AsmData {
        let mut result = self.clone();

        //add new structs
        for (name, unpadded) in parse_data.get_all_structs().iter() {

            result.struct_list.push((name.clone(), unpadded.pad_members(&result)));//add new structs in order
        }

        //when creating local variables, I need struct data beforehand
        let local_variables: Vec<_> = parse_data.get_symbol_table().iter().map(|(a,b)| (a.clone(), b.clone())).collect();

        //overwrite stack variable symbols with local variables (shadowing)
        for (name, var_type) in local_variables {
            let var_size = var_type.memory_size(&result);

            *stack_data += var_size;
            let var_address_offset = stack_data.clone();//increase stack pointer to store extra variable

            let decl = AddressedDeclaration { data_type: var_type.clone(), location: Operand::Mem(MemoryOperand::SubFromBP(var_address_offset.clone())) };//then generate address, as to not overwrite the stack frame

            result.variables.push((name, decl));
        }

        result
    }

    pub fn clone_for_new_loop(&self, break_jump_label: String) -> AsmData {
        let mut result = self.clone();
        result.break_label = Some(break_jump_label);

        result
    }


    pub fn get_variable(&self, name: &str) -> &AddressedDeclaration {
        &self.variables
        .iter()
        .rev()
        .find(|(n, _)| n == name)
        .unwrap()
        .1
    }
    pub fn get_function_return_type(&self) -> &DataType {
        &self.current_function_return_type
    }
    pub fn get_struct(&self, name: &StructIdentifier) -> &StructDefinition {
        &self.struct_list
        .iter()
        .rev()
        .find(|(n,_)| n == name)
        .unwrap()
        .1
    }

    pub fn get_break_label(&self) -> Option<&String> {
        self.break_label.as_ref()
    }
}


/**
 * note this takes a tuple, so that it can be run in an iterator map()
 */
fn generate_global_variable_decl(data: &(String, DataType)) -> (String, AddressedDeclaration) {
    let (var_name, var_type) = data;
    (var_name.to_string(), AddressedDeclaration{ data_type: var_type.clone(), location: Operand::Mem(MemoryOperand::LabelAccess(var_name.to_string())) })
}