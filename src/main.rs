use std::{env, path::PathBuf};

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
mod member_access;
mod args_handling;
pub mod goto_and_labels;
pub mod union_definition;

fn main() {
    let mut input_path = PathBuf::from("test.c");
    let mut output_path = PathBuf::from("a.out");
    let mut do_linking = true;

    let args_vec = env::args().collect::<Vec<String>>();
    let mut args = args_vec.iter().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-c" => {do_linking = false},
            "-o" => {output_path = PathBuf::from(args.next().unwrap())},
            x => {input_path = PathBuf::from(x)}
        }
    }

    compile::compile(&input_path, &output_path, &[], do_linking).unwrap();
}
