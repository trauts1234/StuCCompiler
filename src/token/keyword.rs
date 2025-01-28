
pub enum Keyword{
    BREAK,//break from loop
    CASE,//case: in switch statement
    CONTINUE,//continue in loop
    DEFAULT,//default case in switch statement
    DO,//do while loop
    ELSE,//if else loop
    ENUM,//define an enum
    FOR,//for loop
    GOTO,//jump command
    IF,//if statement
    RETURN,//return from function
    SIZEOF,//get memory size of type
    STRUCT,//define or use a struct
    SWITCH,//switch statement
    TYPEDEF,//define text as a type
    UNION,//same memory for different uses
    WHILE,//while loop
    BOOL,//boolean, warning: is _Bool in c99
}

impl Keyword {
    pub fn try_new(to_token: &str) -> Option<Keyword> {
        match to_token {
            "break" => Some(Self::BREAK),
            "case" => Some(Self::CASE),
            "continue" => Some(Self::CONTINUE),
            "default" => Some(Self::DEFAULT),
            "do" => Some(Self::DO),
            "else" => Some(Self::ELSE),
            "enum" => Some(Self::ENUM),
            "for" => Some(Self::FOR),
            "goto" => Some(Self::GOTO),
            "if" => Some(Self::IF),
            "return" => Some(Self::RETURN),
            "sizeof" => Some(Self::SIZEOF),
            "struct" => Some(Self::STRUCT),
            "switch" => Some(Self::SWITCH),
            "typedef" => Some(Self::TYPEDEF),
            "union" => Some(Self::UNION),
            "while" => Some(Self::WHILE),
            "_Bool" => Some(Self::BOOL),//this one is weird in old versions of C. proper bool, true, false is defined in <stdbool>
            _ => None,
        }
    }
}