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