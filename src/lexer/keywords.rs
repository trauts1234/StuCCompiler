
#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    ENUM,
    IF,
    ELSE,
    FOR,
    WHILE,
    RETURN,
}

impl Keyword {
    pub fn try_new(to_token: &str) -> Option<Keyword> {
        match to_token {
            "enum" => Some(Self::ENUM),
            "if" => Some(Self::IF),
            "else" => Some(Self::ELSE),
            "for" => Some(Self::FOR),
            "while" => Some(Self::WHILE),
            "return" => Some(Self::RETURN),
            _ => None,
        }
    }
}