use std::fmt::Display;

use logos::{Lexer, Logos};

use crate::{data_type::{base_type::{IntegerType, ScalarType}, storage_type::StorageDuration, type_qualifier::TypeQualifier, type_token::TypeInfo}, number_literal::typed_value::NumberLiteral, string_literal::StringLiteral};

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
        NumberLiteral::from(as_string[0].to_string()).cast(&ScalarType::Integer(IntegerType::I32))

    })]
    //normal number literals here
    #[regex(r"0x\.?([pP][+-]|[a-zA-Z0-9\.])*", |x| NumberLiteral::from(x.slice()))]//hex literal
    #[regex(r"\.?[0-9]([eE][+-]|[a-zA-Z0-9\.])*", |x| NumberLiteral::from(x.slice()), priority=1000)]
    NUMBER(NumberLiteral),

    #[token("+=", |_| Punctuator::AdditionCombination)]
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
    #[token("float", |_| TypeInfo::FLOAT)]
    #[token("double", |_| TypeInfo::DOUBLE)]
    TYPESPECIFIER(TypeInfo),

    #[token("auto", |_| StorageDuration::Default)]
    #[token("static", |_| StorageDuration::Static)]
    #[token("extern", |_| StorageDuration::Extern)]
    STORAGESPECIFIER(StorageDuration),

    #[token("const", |_| TypeQualifier::Const)]
    #[token("volatile", |_| TypeQualifier::Volatile)]
    TYPEQUALIFIER(TypeQualifier),

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
    #[token("defined", |_| Keyword::DEFINED)]
    KEYWORD(Keyword),

    #[regex(r"[a-zA-Z_]\w*", |x| x.slice().to_string())]
    IDENTIFIER(String),

    /// This variant should never appear
    #[token("\n")]//newline signals end of parsing this line
    #[regex("//.*\n")]//single line comment completes the line also
    #[regex(r"/\*", consume_comment)]//skip multiline comments
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
                    println!("result: {:?}", result);
                    panic!("error when tokenizing. remainder: {}{}", casted_lexer.slice(), &rem[..usize::min(100, rem.len())]);
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
            Token::TYPEQUALIFIER(x) => x.fmt(f),
            Token::NEWLINE => panic!("tried to Display a newline token")
        }
    }
}

#[derive(Debug, Logos, Clone)]
enum CommentHandling {
    #[token(r"*/", priority=2)]
    CommentEnd,
    #[regex(r"[^\*]", priority=1)]
    CommentText,

    #[token("*", priority=1)]
    CommentAsterisk,
}

fn consume_comment<'a, L>(lex: &mut Lexer<'a, L>) -> logos::Skip
where L: Clone, L: Logos<'a, Extras = (), Source = str, Error = ()>
{

    let new_lex: Lexer<'a, L>= {
        let mut comment_lex: Lexer<'_, CommentHandling> = lex.clone().morph::<CommentHandling>();
        loop {
            match comment_lex.next() {
                Some(Ok(CommentHandling::CommentEnd)) => {
                    //comment is complete
                    break comment_lex.morph();
                },
                Some(Ok(_)) => {},
                Some(Err(_)) => panic!("error parsing comment"),
                None => panic!("unclosed multiline comment")
            }
        }
    };

    *lex = new_lex;
    
    logos::Skip
}