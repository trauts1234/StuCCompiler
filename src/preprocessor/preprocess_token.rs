use logos::{ Logos};

use crate::lexer::token::{consume_comment, Token};

pub struct LineNumbered {
    pub line_num: i32,
    pub data: PreprocessToken
}

#[derive(Clone, Logos, Debug)]
#[logos(skip "[ \\t]+")]
pub enum PreprocessLine {

    #[regex("include[ \\t]*<[^>]+>\n", |x| {
        let slice = x.slice();
        let start_idx = slice.find("<").unwrap() + 1;
        let end_idx = slice.rfind(">").unwrap();
        slice[start_idx..end_idx].to_string()
    })]
    IncludeLib(String),

    #[regex("include[ \\t]*\"[^\"]+\".*\n", |x| {
        let slice = x.slice();
        let start_idx = slice.find("\"").unwrap() + 1;
        let end_idx = slice.rfind("\"").unwrap();
        slice[start_idx..end_idx].to_string()
    })]
    IncludeFile(String),

    #[regex("ifdef.+\n", |x| {
        x.slice()
        .split_once("ifdef").unwrap()
        .1
        .trim()
        .to_string()
    })]
    IfDef(String),

    #[regex("ifndef.+\n", |x| {
        x.slice()
        .split_once("ifndef").unwrap()
        .1
        .trim()
        .to_string()
    })]
    IfNDef(String),

    #[regex("if", Token::parse_logical_line, priority=10)]
    If(Vec<Token>),

    #[regex("pragma.+\n", |x| {
        x.slice()
        .split_once("pragma").unwrap()
        .1
        .trim()
        .to_string()
    })]
    Pragma(String),

     #[regex(r"/\*", consume_comment)]//skip multiline comments

    #[regex("endif.*\n")]
    Endif,

    #[regex("else.*\n",)]
    Else,

    #[regex("elif", Token::parse_logical_line)]
    Elif(Vec<Token>),

    #[regex("define[ \\t]+\\w*", |lex| {
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

    #[regex("undef.+\n", |x| {// #  undef token  \n
        x.slice()
        .split_once("undef")
        .unwrap()
        .1
        .trim()
        .to_string()
    })]
    Undef(String),

    #[regex("error.+\n", |lex| {lex.slice().to_string()})]
    Error(String),

    #[token("\n")]
    NullDirective
}

#[derive(Clone, Logos, Debug)]
#[logos(skip "[ \\t\n]+")]
pub enum PreprocessToken {
    
    #[token("#", |lex| {
        let mut pl_lex = lex.clone().morph::<PreprocessLine>();
        let result = pl_lex.next().unwrap();
        *lex  = pl_lex.morph();

        result
    })]
    Preprocessor(PreprocessLine),

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
    pub fn parse(data: &str) -> Vec<LineNumbered> {
        assert!(data.ends_with("\n"));
        let total_line_count = line_count(data);
        let mut iterator = Self::lexer(data);
        let mut result = Vec::new();

        while let Some(next) = iterator.next() {
            println!("{} {:?}", total_line_count - line_count(iterator.remainder()), next.clone().unwrap());
            match next {
                Ok(x) => result.push(LineNumbered {
                    line_num: total_line_count - line_count(iterator.remainder()),//total lines - lines remaining = line number
                    data: x
                }),

                Err(_) => {
                    let rem = iterator.remainder();
                    //println!("result: {:?}", result);
                    panic!("error when collecting preprocessor lines. remainder: {}{}", iterator.slice(), &rem[..usize::min(100, rem.len())]);
                }
            }
        }

        result
    }
}

/// Note dhat this does not count the last empty line of a file for some reason
fn line_count(data: &str) -> i32 {
    data.lines().count().try_into().unwrap()
}