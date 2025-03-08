use std::env;

mod compound_statement;
mod statement;
mod block_statement;
mod expression;
mod translation_unit;
mod lexer;
pub mod control_flow_statement;
pub mod number_literal;
pub mod type_info;
pub mod asm_boilerplate;
pub mod compile;
pub mod test;
pub mod compilation_error;
pub mod declaration;
mod ast_metadata;
mod memory_size;
mod selection_statement;
pub mod asm_generation;
pub mod preprocessor;
mod function_call;
mod compilation_state;
mod function_definition;
mod function_declaration;
mod string_literal;
mod iteration_statement;

struct CompilationOptions {
    c_file: String,
    out_file: String
}

fn main() {
    let mut options = CompilationOptions{c_file: "main.c".to_string(), out_file: "main".to_string()};

    let args_vec = env::args().collect::<Vec<String>>();
    let mut args = args_vec.iter().skip(1);

    while let Some(arg) = args.next() {
        if arg.starts_with("-o") {
            if arg == "-o" {
                options.out_file = args.next().unwrap().to_string();
            } else {
                options.out_file = arg[2..].to_string();
            }
        } else {
            options.c_file = arg.to_string();
        }
    }


    compile::compile(&options.c_file, &options.out_file).unwrap();
}
