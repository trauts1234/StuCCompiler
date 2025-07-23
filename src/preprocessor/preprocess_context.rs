use std::collections::HashMap;

use crate::{data_type::base_type::IntegerType, lexer::token::Token, number_literal::typed_value::NumberLiteral, string_literal::StringLiteral};

pub struct PreprocessContext {
    defined: HashMap<String, Vec<Token>>,//for simple define
    selection_depth: i32,//how many if statements deep this is
    scan_type: ScanType,//am I skipping code inside a failed #if statement?
    line_counter: i32,
    file_name: StringLiteral,
}

impl PreprocessContext {
    pub fn new(filename: &str) -> PreprocessContext {
        PreprocessContext {
            defined: HashMap::new(),
            selection_depth:0,
            scan_type: ScanType::NORMAL,
            line_counter:1,
            file_name: StringLiteral::new_from_raw(filename.chars())
        }
    }

    pub fn set_scan_type(&mut self, new_type: ScanType) {
        self.scan_type = new_type;
    }
    pub fn get_scan_type(&self) -> ScanType {
        self.scan_type
    }

    pub fn define(&mut self, name: String, value: Vec<Token>) {
        self.defined.insert(name, value);
    }
    pub fn undefine(&mut self, name: &str) {
        self.defined.remove(name);
    }

    pub fn is_defined(&self, name: &str) -> bool {
        self.defined.contains_key(name) ||
        name == "__FILE__" ||
        name == "__LINE__"
    }
    pub fn get_definition(&self, name: &str) -> Option<Vec<Token>> {
        match name {
            "__LINE__" => Some(vec![Token::NUMBER(NumberLiteral::INTEGER { data: self.line_counter.into(), data_type: IntegerType::I32 })]),
            "__FILE__" => Some(vec![Token::STRING(self.file_name.clone())]),
            _ => self.defined.get(name).cloned()
        }
    }

    pub fn selection_depth(&self) -> i32 {
        self.selection_depth
    }
    pub fn inc_selection_depth(&mut self) {
        self.selection_depth += 1;
    }
    pub fn dec_selection_depth(&mut self) {
        self.selection_depth -= 1;
    }
    pub fn set_line_number(&mut self, line: i32) {
        self.line_counter = line;
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum ScanType {
    ///taking all source code
    NORMAL,
    ///Skip all code until *below* this depth
    SKIPPINGBRANCH(i32),
    ///Skip code until you can take a branch at this depth
    FINDINGTRUEBRANCH(i32),
}