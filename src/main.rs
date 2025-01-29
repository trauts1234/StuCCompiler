use translation_unit::TranslationUnit;

mod token;
mod function_definition;
mod compound_statement;
mod statement;
mod block_statement;
mod expression;
mod r_value;
mod l_value;
mod translation_unit;
mod lexer;

fn main() {
    let tu = TranslationUnit::new("test.c");
}
