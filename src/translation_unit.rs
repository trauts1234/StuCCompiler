use crate::{asm_boilerplate, ast_metadata::ASTMetadata, compilation_error::CompilationError, compilation_state::{functions::FunctionList, label_generator::LabelGenerator}, function_declaration::FunctionDeclaration, function_definition::FunctionDefinition, lexer::{lexer::Lexer, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, preprocessor::preprocessor::preprocess_c_file, string_literal::StringLiteral};
use std::{fs::File, io::Write};

#[derive(Debug)]
pub struct TranslationUnit {
    functions: FunctionList,
    string_literals: Vec<StringLiteral>
}

impl TranslationUnit {
    pub fn new(filename: &str) -> Result<TranslationUnit, CompilationError> {

        let data = preprocess_c_file(filename);

        let mut tokens = Vec::new();
        let mut lexer = Lexer::new(&data);
        while let Some(t) = lexer.next_token() {
            tokens.push(t);
        }

        let string_literals: Vec<StringLiteral> = tokens.iter()
            .filter_map(|tok| if let Token::STRING(str_lit) = tok {Some(str_lit)} else {None})//get all strings from the token list
            .cloned()
            .collect();

        println!("{:#?}", tokens);

        let mut token_queue = TokenQueue::new(tokens);
        let mut token_idx = TokenQueueSlice::new();

        let mut funcs = FunctionList::new();

        while token_queue.peek(&token_idx).is_some() {
            if let Some(ASTMetadata{resultant_tree, remaining_slice, extra_stack_used:_}) = FunctionDefinition::try_consume(&mut token_queue, &token_idx, &funcs){
                funcs.add_function(resultant_tree);
                assert!(remaining_slice.index > token_idx.index);
                token_idx = remaining_slice;
            } else if let Some(ASTMetadata { remaining_slice, resultant_tree, extra_stack_used:_ }) = FunctionDeclaration::try_consume(&mut token_queue, &token_idx) {
                funcs.add_declaration(resultant_tree);
                assert!(remaining_slice.index > token_idx.index);
                token_idx = remaining_slice;
            } else {
                return Err(CompilationError::PARSE(format!("unknown remaining data in translation unit: tokens {} and onwards", token_idx.index)));
            }
        }

        Ok(TranslationUnit {
            functions: funcs,
            string_literals:string_literals
        })
    }

    pub fn generate_assembly(&self, output_filename: &str) {
        let mut output_file = File::create(output_filename).unwrap();

        let global_funcs = self.functions.func_declarations_as_slice().iter()
            .filter(|func| func.external_linkage())//only functions with external linkage
            .map(|func| {
                let is_defined = self.functions.get_function_definition(&func.function_name).is_some();
                if is_defined {
                    format!("global {}\n", func.function_name)//global exports my function
                } else {
                    format!("extern {}\n", func.function_name)//extern imports my function
                }
            })
            .collect::<String>();

        let string_literals = self.string_literals.iter()
            .map(|x| format!("{} db {}\n", x.get_label(), x.get_comma_separated_bytes()))
            .collect::<String>();

        let instructions = self.functions.func_definitions_as_slice().iter()
            .map(|x| x.generate_assembly(&mut LabelGenerator::new()))
            .collect::<String>();

        let assembly_code = format!(
"
{}
SECTION .rodata
{}
SECTION .note.GNU-stack ;disable executing the stack
SECTION .text
{}", global_funcs, string_literals, instructions);

        let banned_registers = ["rbx", "r12", "r13", "r14", "r15"];//these ones are callee saved and could cause problems
        assert!(!banned_registers.iter()
            .any(|reg| assembly_code.contains(reg)));//ensure my code does not contain the bad registers

        output_file.write(&assembly_code.into_bytes()).unwrap();
    }
}