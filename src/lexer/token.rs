use crate::{number_literal::NumberLiteral, operator::Operator, type_info::TypeInfo};

#[derive(Debug, Clone, PartialEq)]//for debug printing
pub enum Token {
    //CSTRING(String),
    NUMBER(NumberLiteral),
    OPERATOR(Operator),
    PUNCTUATION(String),
    TYPESPECIFIER(TypeInfo),
    KEYWORD(String),
    IDENTIFIER(String)
}