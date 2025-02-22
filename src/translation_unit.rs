use crate::{asm_boilerplate, ast_metadata::ASTMetadata, compilation_error::CompilationError, compilation_state::{functions::FunctionList, label_generator::LabelGenerator}, function_definition::FunctionDefinition, lexer::{lexer::Lexer, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size::MemoryLayout, preprocessor::preprocessor::preprocess_c_file};
use std::{fs::File, io::Write};

#[derive(Debug)]
pub struct TranslationUnit {
    //variables: Vec<Declaration>,
    functions: FunctionList
}

impl TranslationUnit {
    pub fn new(filename: &str) -> Result<TranslationUnit, CompilationError> {

        let data = preprocess_c_file(filename);

        let mut tokens = Vec::new();
        let mut lexer = Lexer::new(&data);
        while let Some(t) = lexer.next_token() {
            tokens.push(t);
        }

        println!("{:#?}", tokens);

        let mut token_queue = TokenQueue::new(tokens);
        let mut token_idx = TokenQueueSlice::new();

        let mut funcs = FunctionList::new();
        let mut stack_needed = MemoryLayout::new();

        while token_queue.peek(&token_idx).is_some() {
            if let Some(ASTMetadata{resultant_tree, remaining_slice, extra_stack_used}) = FunctionDefinition::try_consume(&mut token_queue, &token_idx, &funcs){
                funcs.add_function(resultant_tree);
                assert!(remaining_slice.index > token_idx.index);
                token_idx = remaining_slice;
                stack_needed += extra_stack_used;
            } else {
                return Err(CompilationError::PARSE(format!("unknown remaining data in translation unit: tokens {} and onwards", token_idx.index)));
            }
        }

        Ok(TranslationUnit {
            functions: funcs
        })
    }

    pub fn generate_assembly(&self, output_filename: &str) {
        let mut output_file = File::create(output_filename).unwrap();

        let instructions = self.functions.funcs_as_slice().iter()
            .map(|x| x.generate_assembly(&mut LabelGenerator::new()))
            .collect::<String>();

        let assembly_code = asm_boilerplate::add_boilerplate(instructions);

        let banned_registers = ["rbx", "r12", "r13", "r14", "r15"];//these ones are callee saved and could cause problems
        assert!(!banned_registers.iter()
            .any(|reg| assembly_code.contains(reg)));//ensure my code does not contain the bad registers

        output_file.write(&assembly_code.into_bytes()).unwrap();
    }
}