use std::collections::HashMap;

use crate::{data_type::base_type::IntegerType, lexer::token::Token, number_literal::typed_value::NumberLiteral, preprocessor::preprocess_token::MacroFunction, string_literal::StringLiteral};

pub struct PreprocessContext {
    defined: HashMap<String, Vec<Token>>,//for simple define
    /// For macro function definitions
    defined_macro_functions: HashMap<String, MacroFunction>,
    selection_depth: i32,//how many if statements deep this is
    scan_type: ScanType,//am I skipping code inside a failed #if statement?
    line_counter: i32,
    line_override: Option<i32>,
    /// Should be used only for the __FILE__ macro as it can be overwritten
    file_name: StringLiteral,
}

impl PreprocessContext {
    pub fn new(filename: &str) -> PreprocessContext {
        PreprocessContext {
            defined: HashMap::new(),
            defined_macro_functions: HashMap::new(),
            selection_depth:0,
            scan_type: ScanType::NORMAL,
            line_counter:1,
            line_override: None,
            file_name: StringLiteral::new_from_raw(filename.chars())
        }
    }

    pub fn set_scan_type(&mut self, new_type: ScanType) {
        self.scan_type = new_type;
    }
    pub fn get_scan_type(&self) -> ScanType {
        self.scan_type
    }

    /// Defines a simple macro
    pub fn define(&mut self, name: String, value: Vec<Token>) {
        self.defined.insert(name, value);
    }
    pub fn undefine(&mut self, name: &str) {
        self.defined.remove(name);
        self.defined_macro_functions.remove(name);
    }

    /// Defines a macro function - still use .undefine to remove it though
    pub fn define_func(&mut self, name: String, func: MacroFunction) {
        self.defined_macro_functions.insert(name, func);
    }

    pub fn get_definition(&self, name: &str) -> Option<Vec<Token>> {
        match name {
            "__LINE__" => {
                let data: i128 = self.line_override.unwrap_or(self.line_counter).into();
                Some(vec![Token::NUMBER(NumberLiteral::INTEGER { data, data_type: IntegerType::I32 })])
            }
            "__FILE__" => Some(vec![Token::STRING(self.file_name.clone())]),
            _ => self.defined.get(name).cloned()
        }
    }
    pub fn get_macro_func(&self, name: &str) -> Option<MacroFunction> {
        self.defined_macro_functions.get(name).cloned()
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
    pub fn override_filename(&mut self, new_filename: StringLiteral) {
        self.file_name = new_filename;
    }
    pub fn override_line_number(&mut self, new_line: i32) {
        self.line_override = Some(new_line);
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