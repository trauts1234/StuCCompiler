
#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    ENUM,
    STRUCT,
    IF,
    ELSE,
    FOR,
    WHILE,
    RETURN,
    BREAK,
    TYPEDEF,
    SIZEOF,
}

impl Keyword {
    pub fn try_new(to_token: &str) -> Option<Keyword> {
        match to_token {
            "enum" => Some(Self::ENUM),
            "struct" => Some(Self::STRUCT),
            "if" => Some(Self::IF),
            "else" => Some(Self::ELSE),
            "for" => Some(Self::FOR),
            "while" => Some(Self::WHILE),
            "return" => Some(Self::RETURN),
            "break" => Some(Self::BREAK),
            "typedef" => Some(Self::TYPEDEF),
            "sizeof" => Some(Self::SIZEOF),
            _ => None,
        }
    }
}