use crate::{function_definition::FunctionDefinition, lexer::lexer::Lexer};
use std::fs;


pub struct TranslationUnit {
    //variables: Vec<Declaration>,
    functions: Vec<FunctionDefinition>
}

impl TranslationUnit {
    pub fn new(filename: &str) -> TranslationUnit {

        let mut data = fs::read_to_string(filename)
            .expect("can't read file")
            .replace("\r\n", "\n")//fix weird newlines
            .replace("\t", " ");//make all whitespace a space character or newline

        data = data.replace("\\\n", "");//remove \ newline, a feature in c

        let mut tokens = Vec::new();
        let mut lexer = Lexer::new(&data);
        while let Some(t) = lexer.next_token() {
            tokens.push(t);
        }

        let mut funcs = Vec::new();

        while tokens.len() > 0 {
            if let Some((next_func_definition, remaining_tokens)) = FunctionDefinition::try_consume_func_definition(&tokens){
                funcs.push(next_func_definition);
                tokens = remaining_tokens;//parse what's left
            } else {
                panic!("unknown remaining data in translation unit:\n{:?}", tokens);
            }
        }

        TranslationUnit {
            functions: funcs
        }
    }
}