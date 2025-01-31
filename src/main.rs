use translation_unit::TranslationUnit;

mod token;
mod function_definition;
mod compound_statement;
mod statement;
mod block_statement;
mod expression;
mod translation_unit;
mod lexer;
pub mod control_flow_statement;

fn main() {
    let tu = TranslationUnit::new("test.c");

    println!("{:#?}", tu);
}
