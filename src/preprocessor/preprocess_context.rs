use std::collections::HashMap;

use crate::lexer::token::Token;

pub struct PreprocessContext {
    defined: HashMap<String, Vec<Token>>,//for simple define
    selection_depth: i32,//how many if statements deep this is
    scan_type: ScanType,//am I skipping code inside a failed #if statement?
}

impl PreprocessContext {
    pub fn new() -> PreprocessContext {
        PreprocessContext {
            defined: HashMap::new(),
            selection_depth:0,
            scan_type: ScanType::NORMAL,
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
        return self.defined.contains_key(name);
    }
    pub fn get_definition(&self, name: &str) -> Option<Vec<Token>> {
        self.defined.get(name).cloned()
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