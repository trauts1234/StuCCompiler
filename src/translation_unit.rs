use crate::{function_definition::FunctionDefinition, lexer::{lexer::Lexer, token_savepoint::TokenQueueLocation, token_walk::TokenQueue}};
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

        let mut token_queue = TokenQueue::new(tokens);
        let mut token_idx = TokenQueueLocation::new();

        let mut funcs = Vec::new();

        while token_queue.peek(&token_idx).is_some() {
            if let Some((next_func_definition, remaining_tokens)) = FunctionDefinition::try_consume(&mut token_queue, &token_idx){
                funcs.push(next_func_definition);
                token_idx = remaining_tokens;
            } else {
                panic!("unknown remaining data in translation unit");
            }
        }

        TranslationUnit {
            functions: funcs
        }
    }
}