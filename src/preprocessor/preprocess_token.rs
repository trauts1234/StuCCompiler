use logos::{ Logos};

use crate::lexer::token::Token;


#[derive(Clone, Logos, Debug)]
#[logos(skip "[ \\t\n]+")]
pub enum PreprocessToken {
    #[regex("#[ \\t]*include[ \\t]*<[^>]+>\n", |x| {
        let slice = x.slice();
        let start_idx = slice.find("<").unwrap() + 1;
        let end_idx = slice.rfind(">").unwrap();
        slice[start_idx..end_idx].to_string()
    })]
    IncludeLib(String),

    #[regex("#[ \\t]*include[ \\t]*\"[^\"]+\".*\n", |x| {
        let slice = x.slice();
        let start_idx = slice.find("\"").unwrap() + 1;
        let end_idx = slice.rfind("\"").unwrap();
        slice[start_idx..end_idx].to_string()
    })]
    IncludeFile(String),

    #[regex("#[ \\t]*ifdef.+\n", |x| {
        x.slice()
        .split_once("ifdef").unwrap()
        .1
        .trim()
        .to_string()
    })]
    IfDef(String),

    #[regex("#[ \\t]*ifndef.+\n", |x| {
        x.slice()
        .split_once("ifndef").unwrap()
        .1
        .trim()
        .to_string()
    })]
    IfNDef(String),

    #[regex("#[ \\t]*if", Token::parse_logical_line, priority=10)]
    If(Vec<Token>),

    #[regex("#[ \\t]*pragma.+\n", |x| {
        x.slice()
        .split_once("pragma").unwrap()
        .1
        .trim()
        .to_string()
    })]
    Pragma(String),

    #[regex("#[ \\t]*endif.*\n")]
    Endif,

    #[regex("#[ \\t]*else.*\n",)]
    Else,

    #[regex("#[ \\t]*elif", Token::parse_logical_line)]
    Elif(Vec<Token>),

    #[regex("#[ \\t]*define[ \\t]+\\w*", |lex| {
        let macro_name = lex.slice()
            .split_once("define").expect("could not find 'define' in a #define macro")
            .1
            .trim()//get the x part of #define x foo
            .to_string();
        // Parse the definition after the macro name
        let macro_definition = Token::parse_logical_line(lex);

        (macro_name, macro_definition)
    })]
    DefineToken((String, Vec<Token>)),// #define x y

    #[regex("#[ \\t]*undef.+\n", |x| {// #  undef token  \n
        x.slice()
        .split_once("undef")
        .unwrap()
        .1
        .trim()
        .to_string()
    })]
    Undef(String),

    #[regex("#[ \\t]*error.+\n", |lex| {lex.slice().to_string()})]
    Error(String),

    #[regex("[^#]", |lex| {
        let start_idx = lex.span().start;
        let text = &lex.source()[start_idx..];
        assert!(!text.starts_with("#"));
        *lex = PreprocessToken::lexer(text);//take back the accidentally consumed character, then parse a line of code
        Token::parse_logical_line(lex)
    }, priority = 1)]
    LineOfCode(Vec<Token>),
}

impl PreprocessToken {
    /// Note: requires trailing newline
    /// 
    /// This still works if comments are present
    pub fn parse(data: &str) -> Vec<Self> {
        assert!(data.ends_with("\n"));
        let mut iterator = Self::lexer(data);
        let mut result = Vec::new();

        while let Some(next) = iterator.next() {
            match next {
                Ok(x) => result.push(x),
                Err(_) => {
                    let rem = iterator.remainder();
                    println!("result: {:?}", result);
                    panic!("error when collecting preprocessor lines. remainder: {}{}", iterator.slice(), &rem[..usize::min(100, rem.len())]);
                }
            }
        }

        result
    }
}