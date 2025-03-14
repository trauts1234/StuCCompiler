use crate::{asm_generation::{asm_comment, asm_line, LogicalRegister, RegisterName}, data_type::{base_type::BaseType, data_type::DataType}, expression::ExprNode};
use std::fmt::Write;

#[derive(Debug, Clone, PartialEq)]
pub struct NumberLiteral {
    unformatted_text: String
}

impl ExprNode for NumberLiteral {
    fn generate_assembly(&self) -> String {
        let mut result = String::new();

        let reg_size = &self.get_data_type().memory_size();//decide how much storage is needed to temporarily store the constant
        asm_comment!(result, "reading number literal: {} via register {}", self.nasm_format(), LogicalRegister::ACC.generate_reg_name(reg_size));

        asm_line!(result, "mov {}, {}", LogicalRegister::ACC.generate_reg_name(reg_size), self.nasm_format());

        result
    }

    fn get_data_type(&self) -> DataType {
        if self.unformatted_text.contains("."){
            panic!("floats not implemented")
        } else {
            DataType::new_from_base_type(&BaseType::I32, &Vec::new())//what about ull suffix?
        }
    }
    
    fn put_lvalue_addr_in_acc(&self) -> String {
        panic!("tried to find memory address of a constant number")
    }
    
    fn clone_self(&self) -> Box<dyn ExprNode> {
        Box::new(self.clone())
    }
}

impl NumberLiteral {
    pub fn try_new(to_token: &str) -> Option<NumberLiteral> {

        if to_token.parse::<f64>().is_ok() {
            Some(NumberLiteral {
                unformatted_text: to_token.to_string()
            })
        } else {
            None
        }
    }

    /**
     * format this number in a way that it can be pasted into a nasm file
     */
    pub fn nasm_format(&self) -> String {
        if self.unformatted_text.contains(".") || self.unformatted_text.contains("e") {//decimal or standard form
            panic!("floats are unsupported");
        }
        if self.unformatted_text.contains("_"){
            panic!("C23 digit separators are unsupported");
        }

        if self.unformatted_text.starts_with("0") && self.unformatted_text.len() > 1 {
            panic!("octal numbers are not supported");
        }
        if self.unformatted_text.starts_with("-0") && self.unformatted_text.len() > 2 {
            panic!("negative octal numbers are not supported")
        }


        self.unformatted_text.to_string()
    }

    pub fn as_usize(&self) -> Option<usize> {
        self.unformatted_text.parse::<usize>().ok()
    }
}