mod function_definition;
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
pub mod operator;
pub mod compile;
pub mod test;
pub mod compilation_error;
pub mod declaration;
mod stack_variables;
mod ast_metadata;
mod memory_size;
mod selection_statement;
mod label_generator;

fn main() {
    compile::compile("test.c", "a").unwrap();
}
