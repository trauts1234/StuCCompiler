use std::process::Command;

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
pub mod memory_type;

fn main() {
    let tu = TranslationUnit::new("test.c");

    println!("{:#?}", tu);

    tu.generate_assembly("a.asm");

    //assemble
    let nasm_status = Command::new("nasm")
        .arg("-f elf64")
        .arg("a.asm")
        .status()
        .expect("Failed to run NASM");
    assert!(nasm_status.success(), "NASM failed to assemble the file");

    //link
    let ld_status = Command::new("ld")
        .arg("a.o")
        .arg("-o")
        .arg("a.out")
        .status()
        .expect("Failed to run linker");
    assert!(ld_status.success(), "Linker failed to produce the binary");
}
