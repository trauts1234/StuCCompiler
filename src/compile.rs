use std::process::Command;

use crate::translation_unit::TranslationUnit;


pub fn compile(input_path: &str, output_name: &str) {
    assert!(!output_name.contains("."), "pleases specify output name, without an extension");
    let assembly_filename = format!("{}.asm", output_name);
    let object_filename = format!("{}.o", output_name);
    let binary_filename = format!("{}.out", output_name);


    let tu = TranslationUnit::new(input_path);

    println!("{:#?}", tu);

    tu.generate_assembly(&assembly_filename);

    //assemble
    let nasm_status = Command::new("nasm")
        .arg("-f elf64")
        .arg("-o")
        .arg(object_filename.clone())
        .arg(assembly_filename)
        .status()
        .expect("Failed to run NASM");
    assert!(nasm_status.success(), "NASM failed to assemble the file");

    //link
    let ld_status = Command::new("ld")
        .arg(object_filename)
        .arg("-o")
        .arg(binary_filename)
        .status()
        .expect("Failed to run linker");
    assert!(ld_status.success(), "Linker failed to produce the binary");
}