use colored::Colorize;
use memory_size::MemorySize;
use stack_management::{baked_stack_frame::BakedSimpleStackFrame, stack_item::StackItemKey};

use crate::{assembly::operation::Label, debugging::IRDisplay};

use super::{register::GPRegister, PTR_SIZE};

#[derive(Clone, Debug)]
pub enum MemoryOperand {
    ///accessing a label, RIP-relative addressed
    LabelAccess(String),
    MemoryAddress {pointer_reg: GPRegister},
    PreviousStackFrame{add_to_rbp: MemorySize},//(remember to add 8 bytes for stack frame and 8 bytes for the return address when creating this enum)
    SubFromBP(StackItemKey),
    AddToSP(MemorySize),
}

impl MemoryOperand {
    pub fn generate_name(&self, stack: &BakedSimpleStackFrame) -> String {
        match self {
            MemoryOperand::SubFromBP(memory_layout) => format!("[rbp-{}]", stack.get(memory_layout).offset_from_bp.size_bytes()),
            MemoryOperand::AddToSP(memory_layout) => format!("[rsp+{}]", memory_layout.size_bytes()),
            MemoryOperand::PreviousStackFrame { add_to_rbp } => format!("[rbp+{}]", add_to_rbp.size_bytes()),
            MemoryOperand::LabelAccess(label) => format!("[rel {}]", label),
            MemoryOperand::MemoryAddress { pointer_reg } => format!("[{}]", pointer_reg.generate_name(PTR_SIZE)),
        }
    }
    pub fn generate_sized_name(&self, size: MemorySize) -> String {
        todo!();
        // format!("{} {}", match size.size_bytes() {
        //     1 => "BYTE PTR",
        //     2 => "WORD PTR",
        //     4 => "DWORD PTR",
        //     8 => "QWORD PTR",
        //     _ => panic!()
        // }, self.generate_name())
    }
}

impl IRDisplay for MemoryOperand {
    fn display_ir(&self) -> String {
        match self {
            MemoryOperand::LabelAccess(label) => format!("[{}]", label),
            MemoryOperand::MemoryAddress { pointer_reg } => format!("[{}]", pointer_reg.generate_name(PTR_SIZE)),
            MemoryOperand::PreviousStackFrame { add_to_rbp } => format!("[rbp+{}]", add_to_rbp),
            MemoryOperand::SubFromBP(stack_item_key) => format!("[{:?}]", stack_item_key),
            MemoryOperand::AddToSP(memory_size) => format!("[rsp+{}]", memory_size.size_bytes()),
        }.blue().to_string()
    }
}