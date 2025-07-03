use std::fmt::Display;

use logos::{Lexer, Logos};

use crate::{data_type::{base_type::BaseType, storage_type::StorageDuration, type_token::TypeInfo}, number_literal::typed_value::NumberLiteral, string_literal::StringLiteral};

use super::{keywords::Keyword, punctuator::Punctuator};

#[derive(Debug, Clone, PartialEq, Logos)]
#[logos(skip "[ \t]")]
pub enum Token {
    #[regex(r#""((\\.)|[^"\\])*""#, |x| {//match a string including
        let slice = x.slice();
        StringLiteral::try_new(&slice[1..slice.len()-1])//remove the speech marks
    })]
    STRING(StringLiteral),

    //char literals get converted to numbers:
    #[regex(r#"'((\\.)|[^\\'])+'"#, |x| {// similar to matching a string as some char literals '\n' can be multi-char
        let slice = x.slice();
        let as_string = StringLiteral::use_escape_sequences(&slice[1..slice.len()-1]);//remove the speech marks, then parse escape sequences to string literal
        assert!(as_string.len() == 2);//char and \0
        //TODO multibyte chars can still go in char literals
        NumberLiteral::from(as_string[0].to_string()).cast(&BaseType::I32)

    })]
    //normal number literals here
    #[regex("((0[xX][a-fA-F0-9]+(\\.[a-fA-F0-9]+[pP][\\+-]?[a-fA-F0-9]+)?)|(0[bB][01]+)|([0-9]+(\\.[0-9]+([eE][\\+-]?[0-9]+)?)?)|(0[0-7]+))[ulfULF]*", |x| NumberLiteral::from(x.slice()))]
    NUMBER(NumberLiteral),

    #[regex(r"\+\+?|\-|\--|\*|/|=|;|~|\||\|\||&&|\^|&|%|!|>>|<<|>|<|<=|>=|==|!=|\}|\{|\[|\]|\(|\)|,|\.(\.\.)?|:|\?", |x| Punctuator::try_new(x.slice()))]
    PUNCTUATOR(Punctuator),

    #[token("int", |_| TypeInfo::INT)]
    #[token("char", |_| TypeInfo::CHAR)]
    #[token("short", |_| TypeInfo::SHORT)]
    #[token("long", |_| TypeInfo::LONG)]
    #[token("_Bool", |_| TypeInfo::_BOOL)]
    #[token("unsigned", |_| TypeInfo::UNSIGNED)]
    #[token("signed", |_| TypeInfo::SIGNED)]
    #[token("void", |_| TypeInfo::VOID)]
    TYPESPECIFIER(TypeInfo),

    #[token("auto", |_| StorageDuration::Default)]
    #[token("static", |_| StorageDuration::Static)]
    #[token("extern", |_| StorageDuration::Extern)]
    STORAGESPECIFIER(StorageDuration),

    #[token("enum", |_| Keyword::ENUM)]
    #[token("struct", |_| Keyword::STRUCT)]
    #[token("if", |_| Keyword::IF)]
    #[token("else", |_| Keyword::ELSE)]
    #[token("for", |_| Keyword::FOR)]
    #[token("while", |_| Keyword::WHILE)]
    #[token("return", |_| Keyword::RETURN)]
    #[token("typedef", |_| Keyword::TYPEDEF)]
    #[token("break", |_| Keyword::BREAK)]
    #[token("sizeof", |_| Keyword::SIZEOF)]
    KEYWORD(Keyword),

    #[regex(r"[a-zA-Z_]\w*", |x| x.slice().to_string())]
    IDENTIFIER(String),

    /// This variant should never appear as it gets removed
    #[token("\n")]
    NEWLINE
}

impl Token {
    /// Parses until a newline, but will still consume multiline strings if required
    pub fn parse_logical_line<'a, L>(lex: &mut Lexer<'a, L>) -> Vec<Self>
    where L: Clone, L: Logos<'a, Extras = (), Source = str, Error = ()>
    {
        let mut casted_lexer: Lexer<'_, Token> = lex.clone().morph::<Token>();

        let mut result = Vec::new();

        'outer: loop {
            match  casted_lexer.next() {
                None |
                Some(Ok(Token::NEWLINE)) => break 'outer,

                Some(Ok(x)) => result.push(x),
                Some(Err(())) => {
                    let rem = casted_lexer.remainder();
                    panic!("error when tokenizing. remainder: {:?}", &rem[..usize::min(100, rem.len())]);
                }
            }
        }

        *lex = casted_lexer.morph();

        result
    }

    pub fn as_punctuator(&self) -> Option<Punctuator> {
        if let Self::PUNCTUATOR(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::STRING(x) => x.fmt(f),
            Token::NUMBER(x) => x.fmt(f),
            Token::PUNCTUATOR(x) => x.fmt(f),
            Token::TYPESPECIFIER(x) => x.fmt(f),
            Token::STORAGESPECIFIER(x) => x.fmt(f),
            Token::KEYWORD(x) => x.fmt(f),
            Token::IDENTIFIER(x) => x.fmt(f),
            Token::NEWLINE => panic!("tried to Display a newline token")
        }
    }
}