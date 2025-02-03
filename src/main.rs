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

fn main() {
    compile::compile("test.c", "a").unwrap();
}
