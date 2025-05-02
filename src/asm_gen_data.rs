use crate::{assembly::operand::{memory_operand::MemoryOperand, Operand}, compilation_state::label_generator::LabelGenerator, data_type::recursive_data_type::DataType, function_declaration::FunctionDeclaration, parse_data::ParseData, struct_definition::{StructDefinition, StructIdentifier}};
use memory_size::MemorySize;

pub trait GetStruct {
    fn get_struct(&self, name: &StructIdentifier) -> &StructDefinition;
}

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
    label_gen: LabelGenerator,
    /// any variable that is accessed via a label, like extern and static variables.
    /// static variables in functions are also stored here
    global_variables: Vec<(String, AddressedDeclaration)>,
    /// all structs declared at a global scope
    global_structs: Vec<(StructIdentifier, StructDefinition)>,
}

impl GlobalAsmData {
    pub fn new(global_parse_data: &ParseData) -> Self {
        let global_variables = global_parse_data.get_symbol_table()
            .iter()
            .map(generate_global_variable_decl)
            .collect();

        //generate a partially complete self, so that structs can be padded using myself
        let mut partial_result = Self {
            function_decls: global_parse_data.func_declarations_as_vec(),
            label_gen: LabelGenerator::default(),
            global_variables,
            global_structs: Vec::new()
        };
        for (name, unpadded) in global_parse_data.get_all_structs() {
            partial_result.global_structs.push((name.clone(), unpadded.pad_members(&partial_result)));
        }

        partial_result
    }

    pub fn get_function_declaration(&self, func_name: &str) -> Option<&FunctionDeclaration> {
        self.function_decls.iter()
        .find(|func| func.function_name == func_name)
    }
    pub fn get_global_variables(&self) -> &[(String, AddressedDeclaration)] {
        &self.global_variables
    }

    pub fn label_gen_mut(&mut self) -> &mut LabelGenerator {
        &mut self.label_gen
    }
}

impl AsmData {
    pub fn for_new_function(global_asm_data: &GlobalAsmData, parse_data: &ParseData, current_function_return_type: DataType, stack_data: &mut MemorySize) -> AsmData {
        let mut result = Self {
            variables: global_asm_data.global_variables.clone(),
            current_function_return_type,
            struct_list: global_asm_data.global_structs.clone(),
            break_label: None,
        };

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

    pub fn get_break_label(&self) -> Option<&String> {
        self.break_label.as_ref()
    }
}

impl GetStruct for AsmData {
    fn get_struct(&self, name: &StructIdentifier) -> &StructDefinition {
        &self.struct_list
        .iter()
        .rev()
        .find(|(n,_)| n == name)
        .unwrap()
        .1
    }
}
impl GetStruct for GlobalAsmData {
    fn get_struct(&self, name: &StructIdentifier) -> &StructDefinition {
        &self.global_structs
        .iter()
        .find(|(n,_)| n == name)
        .unwrap()
        .1
    }
}


/**
 * note this takes a tuple, so that it can be run in an iterator map()
 */
fn generate_global_variable_decl(data: &(String, DataType)) -> (String, AddressedDeclaration) {
    let (var_name, var_type) = data;
    (var_name.to_string(), AddressedDeclaration{ data_type: var_type.clone(), location: Operand::Mem(MemoryOperand::LabelAccess(var_name.to_string())) })
}