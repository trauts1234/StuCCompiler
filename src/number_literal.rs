
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
            println!("warning: octal number detected");
            return self.unformatted_text.to_string() + "o";//nasm octal needs "o"
        }


        self.unformatted_text.to_string()
    }
}