use colored::Colorize;
use memory_size::MemorySize;

use crate::debugging::IRDisplay;

//just a string member, in NASM-friendly format already
#[derive(Clone, Debug)]
pub struct ImmediateValue(pub String);

impl ImmediateValue {
    pub fn generate_name(&self) -> String {
        self.0.clone()
    }
}

impl IRDisplay for ImmediateValue {
    fn display_ir(&self) -> String {
        self.generate_name().cyan().to_string()
    }
}

//extend functionality of memory layout to add extra useful function
pub trait MemorySizeExt {
    /**
     * converts this number as a number of bytes into an immediate value
     */
    fn as_imm(&self) -> ImmediateValue;
}

impl MemorySizeExt for MemorySize {
    fn as_imm(&self) -> ImmediateValue {
        ImmediateValue(self.size_bytes().to_string())
    }
}