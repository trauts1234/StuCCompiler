use std::fmt::Display;


#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    ENUM,
    STRUCT,
    UNION,
    IF,
    ELSE,
    FOR,
    WHILE,
    RETURN,
    BREAK,
    GOTO,
    CONTINUE,
    TYPEDEF,
    SIZEOF,
    DEFINED
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}",
            match self {
                Keyword::ENUM => "enum",
                Keyword::STRUCT => "struct",
                Keyword::UNION => "union",
                Keyword::IF => "if",
                Keyword::ELSE => "else",
                Keyword::FOR => "for",
                Keyword::WHILE => "while",
                Keyword::RETURN => "return",
                Keyword::BREAK => "break",
                Self::CONTINUE => "continue",
                Self::GOTO => "goto",
                Keyword::TYPEDEF => "typedef",
                Keyword::SIZEOF => "sizeof",
                Keyword::DEFINED => "defined",
            }
        )
    }
}