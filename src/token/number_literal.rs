
pub struct NumberLiteral {
    unformatted_text: String
}

impl NumberLiteral {
    pub fn try_new(to_token: &str) -> Option<NumberLiteral> {
        //TODO: match if this is a number

        if to_token.parse::<f64>().is_ok() {
            Some(NumberLiteral {
                unformatted_text: to_token.to_string()
            })
        } else {
            None
        }
    }
}