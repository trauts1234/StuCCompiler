use stack_management::{simple_stack_frame::SimpleStackFrame, stack_item::StackItemKey};

use crate::{asm_gen_data::{AsmData, GlobalAsmData}, assembly::assembly::IRCode, data_type::recursive_data_type::DataType};

pub trait GenerateIR {
    fn generate_ir(&self, asm_data: &AsmData, stack_data: &mut SimpleStackFrame, global_asm_data: &GlobalAsmData) -> (IRCode, Option<StackItemKey>);
}

pub trait GetType {
    fn get_type(&self, asm_data: &AsmData) -> DataType;
}

pub trait GetAddress {
    /// Returns the stack location of a pointer that points to `self`
    /// 
    /// Implementations must not clone the data before pointing to it, as this would defeat the point of getting the address
    fn get_address(&self, asm_data: &AsmData, stack_data: &mut SimpleStackFrame, global_asm_data: &GlobalAsmData) -> (IRCode, StackItemKey);
}