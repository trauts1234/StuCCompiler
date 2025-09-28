use std::{env, path::{Path, PathBuf}};

use clap::{arg, command, Arg, ArgAction};

mod compound_statement;
mod statement;
mod block_statement;
mod expression;
mod translation_unit;
mod lexer;
pub mod control_flow_statement;
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
pub mod generate_ir_traits;

fn main() {

    let matches = command!()
        .arg(
            Arg::new("link with libc")
            .short('l')
            .long("do-linking")
            .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("output file")
            .short('o')
            .long("output")
            .default_value("a.out")
        )
        .arg(
            Arg::new("debug info")
            .short('d')
            .long("debug-info")
        )
        .arg(
            Arg::new("inputs")
            .help("C source files")
            .default_value("test.c")
            .num_args(1)
        )
        .get_matches();

    let do_linking = matches.get_flag("link with libc");
    let output_path = PathBuf::from(matches.get_one::<String>("output file").unwrap());
    let input_path = PathBuf::from(matches.get_one::<String>("inputs").unwrap());
    let debug_out_path = matches.get_one::<String>("debug info").map(|x| PathBuf::from(x));

    compile::compile(&input_path, &output_path, &[], do_linking, debug_out_path).unwrap();
}
