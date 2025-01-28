use super::{cstring::CString, keyword::Keyword, punctuation::Punctuation, type_info::TypeInfo};
pub enum Token {
    PUNCTUATION(Punctuation),
    TYPEINFO(TypeInfo),
    KEYWORD(Keyword),
    STRING(CString),
    OPERATOR,
    
}