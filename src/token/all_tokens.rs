use super::{cstring::CString, keyword::Keyword, number_literal::NumberLiteral, operator::Operator, punctuation::Punctuation, type_info::TypeInfo};
pub enum Token {
    PUNCTUATION(Punctuation),
    TYPEINFO(TypeInfo),
    KEYWORD(Keyword),
    OPERATOR(Operator),
    STRING(CString),
    NUMBERLITERAL(NumberLiteral),
}

impl Token {
    pub fn new(to_token: &str) -> Token{
        if let Some(punc) = Punctuation::try_new(to_token) {
            return Token::PUNCTUATION(punc);
        }

        if let Some(typeinfo) = TypeInfo::try_new(to_token){
            return Token::TYPEINFO(typeinfo);
        }

        if let Some(kw) = Keyword::try_new(to_token){
            return Token::KEYWORD(kw);
        }

        if let Some(op) = Operator::try_new(to_token){
            return Token::OPERATOR(op);
        }

        if let Some(str) = CString::try_new(to_token) {
            return Token::STRING(str);
        }

        if let Some(num) = NumberLiteral::try_new(to_token) {
            return Token::NUMBERLITERAL(num);
        }

        panic!();
    }
}