use crate::{asm_boilerplate, ast_metadata::ASTMetadata, compilation_error::CompilationError, function_definition::FunctionDefinition, label_generator::LabelGenerator, lexer::{lexer::Lexer, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size::MemoryLayout, preprocessor::preprocessor::preprocess};
use std::{fs::File, io::Write};

#[derive(Debug)]
pub struct TranslationUnit {
    //variables: Vec<Declaration>,
    functions: Vec<FunctionDefinition>
}

impl TranslationUnit {
    pub fn new(filename: &str) -> Result<TranslationUnit, CompilationError> {

        let data = preprocess(10, filename)?;

        let mut tokens = Vec::new();
        let mut lexer = Lexer::new(&data);
        while let Some(t) = lexer.next_token() {
            tokens.push(t);
        }

        println!("{:#?}", tokens);

        let mut token_queue = TokenQueue::new(tokens);
        let mut token_idx = TokenQueueSlice::new();

        let mut funcs = Vec::new();
        let mut stack_needed = MemoryLayout::new();

        while token_queue.peek(&token_idx).is_some() {
            if let Some(ASTMetadata{resultant_tree, remaining_slice, extra_stack_used}) = FunctionDefinition::try_consume(&mut token_queue, &token_idx){
                funcs.push(resultant_tree);
                assert!(remaining_slice.index > token_idx.index);
                token_idx = remaining_slice;
                stack_needed += extra_stack_used;
            } else {
                return Err(CompilationError::PARSE(format!("unknown remaining data in translation unit: tokens {} and onwards", token_idx.index)));
            }
        }

        Ok(TranslationUnit {
            functions: funcs
            //TODO use stack_needed
        })
    }

    pub fn generate_assembly(&self, output_filename: &str) {
        let mut output_file = File::create(output_filename).unwrap();

        let instructions = self.functions.iter()
            .map(|x| x.generate_assembly(&mut LabelGenerator::new()))
            .collect::<String>();

        let assembly_code = asm_boilerplate::add_boilerplate(instructions);

        output_file.write(&assembly_code.into_bytes()).unwrap();
    }
}