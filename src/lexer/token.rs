use crate::{number_literal::NumberLiteral, type_info::TypeInfo};

use super::punctuator::Punctuator;

#[derive(Debug, Clone, PartialEq)]//for debug printing
pub enum Token {
    //CSTRING(String),
    NUMBER(NumberLiteral),
    PUNCTUATOR(Punctuator),
    TYPESPECIFIER(TypeInfo),
    KEYWORD(String),
    IDENTIFIER(String)
}

impl Token {
    pub fn as_punctuator(&self) -> Option<Punctuator> {
        if let Self::PUNCTUATOR(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }
}