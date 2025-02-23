use std::collections::HashMap;

pub struct PreprocessContext {
    defined: HashMap<String, String>,//for simple define
    selection_depth: i32,//how many if statements deep this is
    scan_type: ScanType,//am I skipping code inside a failed #if statement?
    pub(crate) inside_string: bool,
    pub(crate) inside_char: bool,
}

impl PreprocessContext {
    pub fn new() -> PreprocessContext {
        PreprocessContext {
            defined: HashMap::new(),
            selection_depth:0,
            scan_type: ScanType::NORMAL,
            inside_string:false,
            inside_char: false
        }
    }

    pub fn set_scan_type(&mut self, new_type: ScanType) {
        self.scan_type = new_type;
    }
    pub fn get_scan_type(&self) -> &ScanType {
        &self.scan_type
    }

    pub fn define(&mut self, name: &str, value: &str) {
        self.defined.insert(name.to_string(), value.to_string());
    }
    pub fn undefine(&mut self, name: &str) {
        self.defined.remove(name);
    }

    pub fn is_defined(&self, name: &str) -> bool {
        return self.defined.contains_key(name);
    }
    pub fn get_definition(&self, name: &str) -> Option<String> {
        self.defined.get(name).map(|x| x.to_string())
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

#[derive(PartialEq)]
pub enum ScanType {
    NORMAL,//taking all source code
    SKIPPINGBRANCH(i32),//selection depth at which you can stop skipping
    FINDINGTRUEBRANCH(i32),//selection depth at which branches are considered
}

fn smallest_option(a: Option<usize>, b: Option<usize>) -> Option<usize> {
    match (a,b) {
        (None,None) => None,
        (Some(x), None) => Some(x),
        (None, Some(x)) => Some(x),
        (Some(x), Some(y)) => Some(x.min(y))
    }
}