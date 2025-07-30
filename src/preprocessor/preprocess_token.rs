use std::{collections::VecDeque, fmt::Debug};

use logos::{ Lexer, Logos};

use crate::{lexer::{punctuator::Punctuator, token::Token}, number_literal::typed_value::NumberLiteral, string_literal::StringLiteral};

pub struct LineNumbered {
    pub line_num: i32,
    pub data: PreprocessToken
}

impl Debug for LineNumbered {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}: {:?}", self.line_num, self.data)
    }
}

#[derive(Debug, Clone, Default)]
pub struct MacroFunction {
    pub params: Vec<String>,
    pub body: Vec<Token>
}

impl MacroFunction {
    pub fn new_from(lex: &mut Lexer<PreprocessToken>) -> (String, Self) {
        
        let macro_name = lex.slice()
            .split_once("define").expect("could not find 'define' in a #define macro")
            .1
            .trim_end_matches("(")//remove the open bracket
            .trim()//get the x part of #define x(y) foo
            .to_string();
    
        let mut tokens_after = VecDeque::from(Token::parse_logical_line(lex));
        let mut result = Self::default();
        
        'param_gather: loop {
            let next = tokens_after.pop_front().unwrap();
            match next {
                Token::PUNCTUATOR(Punctuator::CLOSECURLY) => break 'param_gather,
                Token::IDENTIFIER(param_name) => result.params.push(param_name),
                Token::PUNCTUATOR(Punctuator::COMMA) => {}
                _ => panic!("invalid token when parsing params of a macro function")
            }
        }

        result.body = tokens_after.into();

        (macro_name, result)
    }
}

#[derive(Clone, Logos, Debug)]
#[logos(skip "[ \n]")]
pub enum PreprocessToken {
    
    #[regex("#[ \n]*include *<[^>]+>\n", |x| {
        let slice = x.slice();
        let start_idx = slice.find("<").unwrap() + 1;
        let end_idx = slice.rfind(">").unwrap();
        slice[start_idx..end_idx].to_string()
    })]
    IncludeLib(String),

    #[regex("#[ \n]*include *\"[^\"]+\".*\n", |x| {
        let slice = x.slice();
        let start_idx = slice.find("\"").unwrap() + 1;
        let end_idx = slice.rfind("\"").unwrap();
        slice[start_idx..end_idx].to_string()
    })]
    IncludeFile(String),

    #[regex("#[ \n]*ifdef +", |x| {
        let macro_name = x.remainder().split_once("\n").unwrap().0;
        x.bump(macro_name.len() + 1);//skip the macro name and newline
        macro_name.to_string()
    })]
    IfDef(String),

    #[regex("#[ \n]*ifndef +", |x| {
        let macro_name = x.remainder().split_once("\n").unwrap().0;
        x.bump(macro_name.len() + 1);//skip the macro name and newline
        macro_name.to_string()
    })]
    IfNDef(String),

    #[regex("#[ \n]*if", Token::parse_logical_line, priority=10)]
    If(Vec<Token>),

    #[regex("#[ \n]*pragma.+\n", |x| {
        x.slice()
        .split_once("pragma").unwrap()
        .1
        .trim()
        .to_string()
    })]
    Pragma(String),

    #[regex("#[ \n]*endif.*\n")]
    Endif,

    #[regex("#[ \n]*else.*\n",)]
    Else,

    #[regex("#[ \n]*elif", Token::parse_logical_line)]
    Elif(Vec<Token>),

    #[regex("#[ \n]*define +\\w*", |lex| {
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

    #[regex("#[ \n]*define +\\w+\\(", |lex| {
        MacroFunction::new_from(lex)
    })]
    DefineMacro((String, MacroFunction)),

    #[regex("#[ \n]*undef +", |x| {// #  undef token  \n
        let macro_name = x.remainder().split_once("\n").unwrap().0;
        x.bump(macro_name.len() + 1);//skip the macro name and newline
        macro_name.to_string()
    })]
    Undef(String),

    #[regex("#[ \n]*error.+\n", |lex| {lex.slice().to_string()})]
    Error(String),

    #[regex("#[ \n]*line", |lex| {
        let text = Token::parse_logical_line(lex);
        match &text[..] {
            [Token::NUMBER(new_line_number)] => (new_line_number.clone(), None),
            [Token::NUMBER(new_line_number), Token::STRING(new_filename)] => (new_line_number.clone(), Some(new_filename.clone())),
            x => panic!("invalid tokens after #line: {:?}", &x)
        }
    })]
    LineDirective((NumberLiteral, Option<StringLiteral>)),

    #[regex("#[ ]*\n")]
    NullDirective,

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
            match next {
                Ok(x) => result.push(LineNumbered {
                    line_num: total_line_count - line_count(iterator.remainder()),//total lines - lines remaining = line number
                    data: x
                }),

                Err(_) => {
                    let rem = iterator.remainder();
                    panic!("error when collecting preprocessor lines. remainder: {:?}{:?}", iterator.slice(), &rem[..usize::min(100, rem.len())]);
                }
            }
        }

        result
    }
}

/// Note that this does not count the last empty line of a file for some reason
fn line_count(data: &str) -> i32 {
    data.lines().count().try_into().unwrap()
}