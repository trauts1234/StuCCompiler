use crate::token::type_info::TypeInfo;


#[derive(Debug, Clone, PartialEq)]//for debug printing
pub enum Token {
    //CSTRING(String),
    NUMBER(String),
    //OPERATOR(String),
    PUNCTUATION(String),
    TYPESPECIFIER(TypeInfo),
    KEYWORD(String),
    IDENTIFIER(String)
}