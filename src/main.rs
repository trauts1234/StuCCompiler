use translation_unit::TranslationUnit;

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

fn main() {
    let tu = TranslationUnit::new("test.c");

    println!("{:#?}", tu);

    tu.generate_assembly("a.asm");
}
