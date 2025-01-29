use crate::token::identifier::Identifier;

use super::{cstring::CString, keyword::Keyword, number_literal::NumberLiteral, operator::Operator, punctuation::Punctuation, type_info::TypeInfo};
pub enum Token {
    PUNCTUATION(Punctuation),
    TYPEINFO(TypeInfo),
    KEYWORD(Keyword),
    OPERATOR(Operator),
    STRING(CString),
    NUMBERLITERAL(NumberLiteral),
    IDENTIFIER(Identifier)
}

impl Token {
    pub fn try_new(to_token: &str) -> Option<Token> {

        if let Some(punc) = Punctuation::try_new(to_token) {
            return Some(Token::PUNCTUATION(punc));
        }

        if let Some(typeinfo) = TypeInfo::try_new(to_token){
            return Some(Token::TYPEINFO(typeinfo));
        }

        if let Some(kw) = Keyword::try_new(to_token){
            return Some(Token::KEYWORD(kw));
        }

        if let Some(op) = Operator::try_new(to_token){
            return Some(Token::OPERATOR(op));
        }

        if let Some(str) = CString::try_new(to_token) {
            return Some(Token::STRING(str));
        }

        if let Some(num) = NumberLiteral::try_new(to_token) {
            return Some(Token::NUMBERLITERAL(num));
        }

        if let Some(ident) = Identifier::try_new(to_token) {
            return Some(Token::IDENTIFIER(ident));//this one must be last as almost anything can be an identifier
        }

        None
    }

    pub fn is_non_identifier_token(potential: &str) -> bool {
        if let Some(tok) = Self::try_new(potential) {
            match tok{
                Self::IDENTIFIER(_) => false,
                _ => true,
            }
        } else {
            false
        }
    }
}