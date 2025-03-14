use crate::{ast_metadata::ASTMetadata, compilation_error::CompilationError, compilation_state::{functions::FunctionList, label_generator::LabelGenerator}, declaration::InitialisedDeclaration, function_declaration::FunctionDeclaration, function_definition::FunctionDefinition, lexer::{lexer::Lexer, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, preprocessor::preprocessor::preprocess_c_file, scope_data::ScopeData, string_literal::StringLiteral};
use std::{fs::File, io::Write};

pub struct TranslationUnit {
    functions: FunctionList,
    string_literals: Vec<StringLiteral>,
    global_variables: Vec<InitialisedDeclaration>
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
        let mut global_vars = Vec::new();

        while !token_queue.no_remaining_tokens(&token_idx) {

            let mut scope_data = ScopeData::make_empty();

            if let Some(ASTMetadata{resultant_tree, remaining_slice, extra_stack_used:_}) = FunctionDefinition::try_consume(&mut token_queue, &token_idx, &funcs, &mut scope_data){
                funcs.add_function(resultant_tree);
                assert!(remaining_slice.index > token_idx.index);
                token_idx = remaining_slice;
            } else if let Some(ASTMetadata { remaining_slice, resultant_tree, extra_stack_used:_ }) = FunctionDeclaration::try_consume(&mut token_queue, &token_idx, &mut scope_data) {
                funcs.add_declaration(resultant_tree);
                assert!(remaining_slice.index > token_idx.index);
                token_idx = remaining_slice;
            } else if let Some(ASTMetadata { remaining_slice,mut resultant_tree, extra_stack_used:_ }) = InitialisedDeclaration::try_consume(&mut token_queue, &token_idx, &FunctionList::new(), &mut scope_data) {
                //functions are not passed to the decl consumer as the decl has to be a compile time constant
                global_vars.append(&mut resultant_tree);
                token_idx = remaining_slice;
            } else {
                return Err(CompilationError::PARSE(format!("unknown remaining data in translation unit: tokens {} and onwards", token_idx.index)));
            }
            println!("{:?}", scope_data);
        }

        Ok(TranslationUnit {
            functions: funcs,
            string_literals:string_literals,
            global_variables: global_vars
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

        if self.global_variables.len() != 0 {
            panic!("not implemented: calculating compile time constants for global variables");
        }

        let mut label_generator = LabelGenerator::new();

        let instructions = self.functions.func_definitions_as_slice().iter()
            .map(|x| x.generate_assembly(&mut label_generator))
            .collect::<String>();

        let assembly_code = format!(
"
{}
SECTION .rodata
{}
SECTION .note.GNU-stack ;disable executing the stack
SECTION .text
{}",global_funcs, string_literals, instructions);

        let banned_registers = ["rbx", "r12", "r13", "r14", "r15"];//these ones are callee saved and could cause problems
        assert!(!banned_registers.iter()
            .any(|reg| assembly_code.contains(reg)));//ensure my code does not contain the bad registers

        output_file.write(&assembly_code.into_bytes()).unwrap();
    }
}