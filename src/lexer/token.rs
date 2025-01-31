use crate::{number_literal::NumberLiteral, type_info::TypeInfo};

#[derive(Debug, Clone, PartialEq)]//for debug printing
pub enum Token {
    //CSTRING(String),
    NUMBER(NumberLiteral),
    //OPERATOR(String),
    PUNCTUATION(String),
    TYPESPECIFIER(TypeInfo),
    KEYWORD(String),
    IDENTIFIER(String)
}