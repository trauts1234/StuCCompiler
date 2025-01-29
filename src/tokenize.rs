use crate::token::all_tokens::Token;

struct TokenizeData{
    all_text: String,
    slice_end: usize//index one past the end of the current token
    //warning: parsing after a typedef is difficult
}


pub fn parse_to_tokens(text: &str) -> Vec<Token> {
    
}