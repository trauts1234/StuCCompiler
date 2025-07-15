use colored::Colorize;
use memory_size::MemorySize;

use crate::{data_type::base_type::FloatType, debugging::IRDisplay, number_literal::typed_value::NumberLiteral};

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
pub trait ToImmediate {
    /**
     * converts this number as a number of bytes into an immediate value
     */
    fn as_imm(&self) -> ImmediateValue;
}

impl ToImmediate for MemorySize {
    fn as_imm(&self) -> ImmediateValue {
        ImmediateValue(self.size_bytes().to_string())
    }
}
impl ToImmediate for NumberLiteral {
    fn as_imm(&self) -> ImmediateValue {
        match self {
            NumberLiteral::INTEGER { data, ..} => ImmediateValue(data.to_string()),
            NumberLiteral::FLOAT { data, data_type } => 
                match data_type {
                    FloatType::F32 => ImmediateValue((*data as f32).to_bits().to_string()),//raw bitpattern
                    FloatType::F64 => ImmediateValue(data.to_bits().to_string()),//raw bitpattern
                },
        }
    }
}