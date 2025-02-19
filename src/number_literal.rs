use crate::{declaration::Declaration, type_info::{DataType, TypeInfo}};


#[derive(Debug, Clone, PartialEq)]
pub struct NumberLiteral {
    unformatted_text: String
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

    pub fn get_data_type(&self) -> Declaration {
        if self.unformatted_text.contains("."){
            panic!("floats not implemented")
        } else {
            Declaration {
                data_type: DataType {
                    type_info: vec![TypeInfo::INT],
                    modifiers: Vec::new(),
                },
                name: String::new(),
            }
        }
    }

    pub fn as_usize(&self) -> Option<usize> {
        self.unformatted_text.parse::<usize>().ok()
    }
}