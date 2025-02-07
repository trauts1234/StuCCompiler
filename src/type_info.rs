
#[derive(Debug, Clone, PartialEq)]
pub enum TypeInfo{
    INT,
    //CHAR,//TODO
    //missing some, should have "static", and other bits that suggest the type of a variable
}

impl TypeInfo {
    pub fn try_new(to_token: &str) -> Option<TypeInfo>{
        match to_token {
            "int" => Some(Self::INT),
            //"char" => Some(Self::CHAR),
            _ => None
        }
    }
}