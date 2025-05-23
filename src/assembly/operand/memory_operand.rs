use colored::Colorize;
use memory_size::MemorySize;

use crate::debugging::IRDisplay;

use super::{register::Register, PTR_SIZE};

#[derive(Clone, Debug)]
pub enum MemoryOperand {
    ///accessing a label, RIP-relative addressed
    LabelAccess(String),
    MemoryAddress {pointer_reg: Register},//TODO allow simple expressions, like in LEA instruction. maybe this would remove need for AddToSP/SubFromBP variants
    PreviousStackFrame{add_to_rbp: MemorySize},//(remember to add 8 bytes for stack frame and 8 bytes for the return address when creating this enum)
    SubFromBP(MemorySize),
    AddToSP(MemorySize),
}

impl MemoryOperand {
    pub fn generate_name(&self) -> String {
        match self {
            MemoryOperand::SubFromBP(memory_layout) => format!("[rbp-{}]", memory_layout.size_bytes()),
            MemoryOperand::AddToSP(memory_layout) => format!("[rsp+{}]", memory_layout.size_bytes()),
            MemoryOperand::PreviousStackFrame { add_to_rbp } => format!("[rbp+{}]", add_to_rbp.size_bytes()),
            MemoryOperand::LabelAccess(label) => format!("[rel {}]", label),
            MemoryOperand::MemoryAddress { pointer_reg } => format!("[{}]", pointer_reg.generate_name(PTR_SIZE)),
        }
    }
}

impl IRDisplay for MemoryOperand {
    fn display_ir(&self) -> String {
        self.generate_name().blue().to_string()
    }
}