use crate::{asm_boilerplate, compilation_error::CompilationError, function_definition::FunctionDefinition, lexer::{lexer::Lexer, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}};
use std::{fs::{self, File}, io::Write};

#[derive(Debug)]
pub struct TranslationUnit {
    //variables: Vec<Declaration>,
    functions: Vec<FunctionDefinition>
}

impl TranslationUnit {
    pub fn new(filename: &str) -> Result<TranslationUnit, CompilationError> {

        let data = fs::read_to_string(filename)?;

        let mut tokens = Vec::new();
        let mut lexer = Lexer::new(&data);
        while let Some(t) = lexer.next_token() {
            tokens.push(t);
        }

        let mut token_queue = TokenQueue::new(tokens);
        let mut token_idx = TokenQueueSlice::new();

        let mut funcs = Vec::new();

        while token_queue.peek(&token_idx).is_some() {
            if let Some((next_func_definition, remaining_tokens)) = FunctionDefinition::try_consume(&mut token_queue, &token_idx){
                funcs.push(next_func_definition);
                token_idx = remaining_tokens;
            } else {
                return Err(CompilationError::PARSE("unknown remaining data in translation unit".to_string()));
            }
        }

        Ok(TranslationUnit {
            functions: funcs
        })
    }

    pub fn generate_assembly(&self, output_filename: &str) {
        let mut output_file = File::create(output_filename).unwrap();

        let instructions = self.functions.iter()
            .map(|x| x.generate_assembly())
            .collect::<String>();

        let assembly_code = asm_boilerplate::add_boilerplate(instructions);

        output_file.write(&assembly_code.into_bytes()).unwrap();
    }
}