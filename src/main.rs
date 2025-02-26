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
fn main() {
    compile::compile("test.c", "a").unwrap();
}
