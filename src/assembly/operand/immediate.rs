use memory_size::MemorySize;
use crate::{data_type::base_type::IntegerType, number_literal::typed_value::NumberLiteral};

//extend functionality of memory layout to add extra useful function
pub trait ToImmediate {
    /**
     * converts this number as a number of bytes into an immediate value
     */
    fn as_imm(&self) -> NumberLiteral;
}

impl ToImmediate for MemorySize {
    fn as_imm(&self) -> NumberLiteral {
        NumberLiteral::INTEGER{ data: self.size_bytes().into(), data_type: IntegerType::U64}
    }
}