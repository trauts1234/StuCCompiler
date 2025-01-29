
#[derive(Debug)]//for debug printing
pub enum Token {
    //CSTRING(String),
    NUMBER(String),
    //OPERATOR(String),
    PUNCTUATION(String),
    TYPESPECIFIER(String),
    KWORDORIDENT(String),//keyword or identifier
}