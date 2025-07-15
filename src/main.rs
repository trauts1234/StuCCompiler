use std::{env, path::{Path, PathBuf}, str::FromStr};

mod compound_statement;
mod statement;
mod block_statement;
mod expression;
mod translation_unit;
mod lexer;
pub mod control_flow_statement;
pub mod asm_boilerplate;
pub mod compile;
pub mod test;
pub mod compilation_error;
pub mod declaration;
mod ast_metadata;
mod selection_statement;
pub mod preprocessor;
mod function_call;
mod compilation_state;
mod function_definition;
mod function_declaration;
mod string_literal;
mod iteration_statement;
pub mod data_type;
mod binary_expression;
mod enum_definition;
mod constexpr_parsing;
mod global_var_declaration;
mod struct_definition;
mod parse_data;
mod asm_gen_data;
mod expression_visitors;
pub mod assembly;
pub mod initialised_declaration;
mod cast_expr;
mod typedef;
pub mod array_initialisation;
mod number_literal;
mod debugging;
mod struct_member_access;
mod args_handling;
mod stack_allocation;

struct CompilationOptions {
    c_file: PathBuf,
    link_with: Vec<PathBuf>,
    out_file: PathBuf
}

fn main() {
    let mut options = CompilationOptions{c_file: PathBuf::from_str("test.c").unwrap(), out_file: PathBuf::from_str("a.out").unwrap(), link_with: Vec::new()};

    let args_vec = env::args().collect::<Vec<String>>();
    let mut args = args_vec.iter().skip(1);

    while let Some(arg) = args.next() {
        if arg.starts_with("-o") {
            if arg == "-o" {
                options.out_file = PathBuf::from_str(args.next().unwrap()).unwrap();
            } else {
                options.out_file = PathBuf::from_str(&arg[2..]).unwrap();
            }
        } else {
            let file_path = PathBuf::from_str(arg).unwrap();
            match arg.split_once(".").unwrap().1 {
                "c" => options.c_file = file_path,
                "o" => options.link_with.push(file_path),
                _ => panic!()
            }
        }
    }

    let link_with: Vec<&Path> = options.link_with.iter().map(|p| p.as_path()).collect();
    compile::compile(&options.c_file, &options.out_file, &link_with).unwrap();
}
