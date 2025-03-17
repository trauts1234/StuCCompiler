#[derive(Debug, Clone, PartialEq)]
pub enum TypeInfo{
    INT,
    CHAR,
    _BOOL,
    UNSIGNED,
    LONG,
    SHORT,
    EXTERN,
    VOID,

    VaArg,//varadic arg has a special type
    //missing some, should have "static", and other bits that suggest the type of a variable
}

impl TypeInfo {
    pub fn try_new(to_token: &str) -> Option<TypeInfo>{
        match to_token {
            "unsigned" => Some(Self::UNSIGNED),
            "int" => Some(Self::INT),
            "long" => Some(Self::LONG),
            "short" => Some(Self::SHORT),
            "char" => Some(Self::CHAR),
            "_Bool" => Some(Self::_BOOL),
            "extern" => Some(Self::EXTERN),
            "void" => Some(Self::VOID),
            _ => None
        }
    }
}