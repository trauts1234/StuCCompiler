use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeInfo{
    INT,
    CHAR,
    _BOOL,
    UNSIGNED,
    SIGNED,
    LONG,
    SHORT,
    VOID,

    VaArg,//varadic arg has a special type
}

impl TypeInfo {
    pub fn try_new(to_token: &str) -> Option<TypeInfo>{
        match to_token {
            "unsigned" => Some(Self::UNSIGNED),
            "signed" => Some(Self::SIGNED),
            "int" => Some(Self::INT),
            "long" => Some(Self::LONG),
            "short" => Some(Self::SHORT),
            "char" => Some(Self::CHAR),
            "_Bool" => Some(Self::_BOOL),
            "void" => Some(Self::VOID),
            _ => None
        }
    }
}

impl Display for TypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}",
            match self {
                TypeInfo::INT => "int",
                TypeInfo::CHAR => "char",
                TypeInfo::_BOOL => "bool",
                TypeInfo::UNSIGNED => "unsigned",
                TypeInfo::SIGNED => "signed",
                TypeInfo::LONG => "long",
                TypeInfo::SHORT => "short",
                TypeInfo::VOID => "void",
                TypeInfo::VaArg => "...",
            }
        )
    }
}