use crate::{asm_gen_data::AsmData, ast_metadata::ASTMetadata, compilation_error::CompilationError, compilation_state::{functions::FunctionList, label_generator::LabelGenerator}, function_declaration::FunctionDeclaration, function_definition::FunctionDefinition, global_var_declaration::GlobalVariable, lexer::{lexer::Lexer, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, parse_data::ParseData, preprocessor::preprocessor::preprocess_c_file, string_literal::StringLiteral};
use std::{fs::File, io::Write};

pub struct TranslationUnit {
    functions: FunctionList,
    global_scope_data: ParseData,
    string_literals: Vec<StringLiteral>,
    global_variables: Vec<GlobalVariable>
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

        let mut functions = FunctionList::new();
        let mut global_variables = Vec::new();
        let mut scope_data = ParseData::make_empty();

        while !token_queue.no_remaining_tokens(&token_idx) {

            if let Some(ASTMetadata{resultant_tree, remaining_slice, extra_stack_used:_}) = FunctionDefinition::try_consume(&mut token_queue, &token_idx, &functions, &scope_data){
                functions.add_function(&mut scope_data, resultant_tree);
                assert!(remaining_slice.index > token_idx.index);
                token_idx = remaining_slice;
            } else if let Some(ASTMetadata { remaining_slice, resultant_tree, extra_stack_used:_ }) = FunctionDeclaration::try_consume(&mut token_queue, &token_idx, &mut scope_data.clone_for_new_scope()) {
                //do I need to save the clone of scope data I passed? probably not
                scope_data.add_declaration(resultant_tree);
                assert!(remaining_slice.index > token_idx.index);
                token_idx = remaining_slice;
            } else if let Some(ASTMetadata { remaining_slice,mut resultant_tree, extra_stack_used:_ }) = GlobalVariable::try_consume(&mut token_queue, &token_idx, &mut scope_data) {
                global_variables.append(&mut resultant_tree);
                token_idx = remaining_slice;
            } else {
                return Err(CompilationError::PARSE(format!("unknown remaining data in translation unit: tokens {} and onwards", token_idx.index)));
            }
        }

        Ok(TranslationUnit {
            functions,
            global_scope_data: scope_data,
            string_literals,
            global_variables
        })
    }

    pub fn generate_assembly(&self, output_filename: &str) {
        let mut output_file = File::create(output_filename).unwrap();

        let global_funcs = self.global_scope_data.func_declarations_as_vec().iter()
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

        let global_vars = self.global_variables.iter()
            .map(|x| x.generate_assembly())
            .collect::<String>();

        let mut label_generator = LabelGenerator::new();
        let asm_data = AsmData::new_for_global_scope(&self.global_scope_data);//no return type for a global scope

        let instructions = self.functions.func_definitions_as_slice().iter()
            .map(|x| x.generate_assembly(&mut label_generator, &asm_data))
            .collect::<String>();

        let assembly_code = format!(
"
{}
SECTION .rodata
{}
SECTION .data
{}
SECTION .note.GNU-stack ;disable executing the stack
SECTION .text
{}",global_funcs, string_literals, global_vars, instructions);

        let banned_registers = ["rbx", "r12", "r13", "r14", "r15"];//these ones are callee saved and could cause problems
        assert!(!banned_registers.iter()
            .any(|reg| assembly_code.contains(reg)));//ensure my code does not contain the bad registers

        output_file.write(&assembly_code.into_bytes()).unwrap();
    }
}