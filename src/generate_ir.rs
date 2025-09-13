use stack_management::{simple_stack_frame::SimpleStackFrame, stack_item::StackItemKey};

use crate::{asm_gen_data::{AsmData, GlobalAsmData}, assembly::assembly::Assembly};

pub trait GenerateIR {
    fn generate_ir(&self, asm_data: &AsmData, stack_data: &mut SimpleStackFrame, global_asm_data: &GlobalAsmData) -> (Assembly, Option<StackItemKey>);
}