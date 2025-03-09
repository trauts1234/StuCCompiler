use crate::{data_type::type_token::TypeInfo, number_literal::NumberLiteral, string_literal::StringLiteral};

use super::punctuator::Punctuator;

#[derive(Debug, Clone, PartialEq)]//for debug printing
pub enum Token {
    STRING(StringLiteral),
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