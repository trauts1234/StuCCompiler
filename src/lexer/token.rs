use std::fmt::Display;

use crate::{data_type::{storage_type::StorageDuration, type_token::TypeInfo}, number_literal::typed_value::NumberLiteral, string_literal::StringLiteral};

use super::{keywords::Keyword, punctuator::Punctuator};

#[derive(Debug, Clone, PartialEq)]//for debug printing
pub enum Token {
    STRING(StringLiteral),
    NUMBER(NumberLiteral),
    PUNCTUATOR(Punctuator),
    TYPESPECIFIER(TypeInfo),
    STORAGESPECIFIER(StorageDuration),
    KEYWORD(Keyword),
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

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::STRING(x) => x.fmt(f),
            Token::NUMBER(x) => x.fmt(f),
            Token::PUNCTUATOR(x) => x.fmt(f),
            Token::TYPESPECIFIER(x) => x.fmt(f),
            Token::STORAGESPECIFIER(x) => x.fmt(f),
            Token::KEYWORD(x) => x.fmt(f),
            Token::IDENTIFIER(x) => x.fmt(f),
        }
    }
}