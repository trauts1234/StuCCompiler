use colored::Colorize;
use stack_management::simple_stack_frame::SimpleStackFrame;

use crate::{asm_gen_data::GlobalAsmData, assembly::{assembly::Assembly, assembly_file::AssemblyFile}, ast_metadata::ASTMetadata, compilation_error::CompilationError, compilation_state::{functions::FunctionList}, data_type::storage_type::StorageDuration, debugging::{ASTDisplay, IRDisplay}, function_declaration::FunctionDeclaration, function_definition::FunctionDefinition, global_var_declaration::GlobalVariable, lexer::{ token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, parse_data::ParseData, preprocessor::preprocessor::preprocess_c_file, string_literal::StringLiteral, typedef::Typedef};
use std::{collections::HashSet, fs::File, io::Write, path::Path};

pub struct TranslationUnit {
    functions: FunctionList,
    global_scope_data: ParseData,
    string_literals: HashSet<StringLiteral>,
    global_variables: Vec<GlobalVariable>
}

impl TranslationUnit {
    pub fn new(filename: &Path) -> Result<TranslationUnit, CompilationError> {

        let tokens = preprocess_c_file(filename);

        println!("{:?}", tokens);

        let string_literals: HashSet<StringLiteral> = tokens.iter()
            .filter_map(|tok| if let Token::STRING(str_lit) = tok {Some(str_lit)} else {None})//get all strings from the token list
            .cloned()
            .collect();

        let mut token_queue = TokenQueue::new(tokens);
        let mut token_idx = TokenQueueSlice::new();

        let mut functions = FunctionList::new();
        let mut global_variables = Vec::new();
        let mut scope_data = ParseData::make_empty();

        while !token_queue.no_remaining_tokens(&token_idx) {

            if let Some(ASTMetadata{resultant_tree, remaining_slice }) = FunctionDefinition::try_consume(&mut token_queue, &token_idx, &scope_data){
                functions.add_function(&mut scope_data, resultant_tree);
                assert!(remaining_slice.index > token_idx.index);
                token_idx = remaining_slice;
            } else if let Some(ASTMetadata { remaining_slice, resultant_tree }) = FunctionDeclaration::try_consume(&mut token_queue, &token_idx, &mut scope_data.clone_for_new_scope()) {
                //do I need to save the clone of scope data I passed? probably not
                scope_data.add_declaration(resultant_tree);
                assert!(remaining_slice.index > token_idx.index);
                token_idx = remaining_slice;
            } else if let Some(ASTMetadata { remaining_slice,mut resultant_tree }) = GlobalVariable::try_consume(&mut token_queue, &token_idx, &mut scope_data) {
                global_variables.append(&mut resultant_tree);
                token_idx = remaining_slice;
            } else if let Some(ASTMetadata { remaining_slice, resultant_tree: (name, new_def, storage_duration) }) = Typedef::try_consume(&token_queue, &token_idx, &mut scope_data) {
                scope_data.add_typedef(name, new_def);
                token_idx = remaining_slice;
            } else {
                return Err(CompilationError::PARSE(format!("unknown remaining data in translation unit: tokens {} and onwards:\n{}", token_idx.index, token_queue.display_slice(&token_idx))));
            }
        }

        Ok(TranslationUnit {
            functions,
            global_scope_data: scope_data,
            string_literals,
            global_variables
        })
    }

    pub fn generate_assembly(&self, output_filename: &Path) {
        let mut output_file = File::create(output_filename).unwrap();
        let mut global_asm_data = GlobalAsmData::new(&self.global_scope_data);

        //get the names of global and extern functions
        let (global_funcs, extern_funcs): (Vec<_>, Vec<_>) = self
        .global_scope_data
        .func_declarations_as_vec()
        .iter()
        .filter(|func| func.external_linkage())//only functions with external linkage
        .map(|func| func.function_name.clone())//get function name
        .partition(|func_name| self.functions.get_function_definition(&func_name).is_some());//separate global and extern function declarations
        //get declarations of global and extern vars
        let (global_vars, extern_vars): (Vec<_>, Vec<_>) = self
        .global_variables
        .iter()//go through global variables
        .filter(|x| *x.storage_class() != StorageDuration::Static)//remove static variables
        .partition(|x| *x.storage_class() != StorageDuration::Extern);//split by whether it is extern
        //generate the names of labels that need to be marked global or extern
        let global_labels: Vec<_> = global_vars.iter()
            .map(|x| x.var_name().to_owned())
            .chain(global_funcs.into_iter())
            .collect();
        let extern_labels: Vec<_> = extern_vars.iter()
            .map(|x| x.var_name().to_owned())
            .chain(extern_funcs.into_iter())
            .collect();

        let string_literals = self.string_literals.iter()
            .map(|x| format!("{} db {}\n", x.get_label(), x.get_comma_separated_bytes()))
            .collect::<Vec<_>>();

        let global_vars_init = self.global_variables.iter()
            .filter(|x| *x.storage_class() != StorageDuration::Extern)//extern variables must not be defined
            .map(|x| x.generate_assembly(&global_asm_data))
            .collect::<Vec<_>>();

        let instructions = self.generate_fn_asm(&mut global_asm_data);

        let assembly_file = AssemblyFile::builder()
        .global_label_lines(global_labels)
        .extern_label_lines(extern_labels)
        .string_literal_lines(string_literals)
        .global_variable_init(global_vars_init)
        .functions(instructions)
        .build();

        let assembly_code = assembly_file.to_nasm_file();

        let banned_registers = ["rbx", "r12", "r13", "r14", "r15"];//these ones are callee saved and could cause problems
        assert!(!banned_registers.iter()
            .any(|reg| assembly_code.contains(reg)));//ensure my code does not contain the bad registers

        output_file.write(&assembly_code.into_bytes()).unwrap();
    }

    fn generate_fn_asm(&self, global_asm_data: &mut GlobalAsmData) -> Vec<(Assembly, SimpleStackFrame)> {

        self.functions.func_definitions_as_slice().iter()
        .map(|x| x.generate_assembly(global_asm_data))
        .collect()
    }
}

impl ASTDisplay for TranslationUnit {
    fn display_ast(&self, f: &mut crate::debugging::TreeDisplayInfo) {
        for func in self.functions.func_definitions_as_slice() {
            func.display_ast(f);
        }
    }
}

impl IRDisplay for TranslationUnit {
    fn display_ir(&self) -> String {
        let mut global_asm_data = GlobalAsmData::new(&self.global_scope_data);

        let str_literals = format!(
            "{}\n{}",
            "string literals:".purple(),
            self.string_literals
                .iter()
                .map(|x| format!("const string {} = {}", x.get_label(), x))
                .collect::<Vec<_>>()
                .join("\n")
        );

        let global_variables = format!(
            "{}\n{}",
            "global variables:".purple(),
            self.global_variables
                .iter()
                .map(|x| x.display_ir())
                .collect::<Vec<_>>()
                .join("\n")
        );

        let asm = self.generate_fn_asm(&mut global_asm_data)
        .iter()
        .map(|(x, _)| x.display_ir())
        .collect:: <Vec<_>>()
        .join("\n");

        format!("{}\n{}\n{}", str_literals, global_variables, asm)
        
    }
}