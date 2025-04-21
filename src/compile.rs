use std::{path::Path, process::Command};

use crate::{compilation_error::CompilationError, debugging::{ASTDisplay, IRDisplay, TreeDisplayInfo}, translation_unit::TranslationUnit};


pub fn compile(input_path: &Path, output_name: &Path, link_with: &[&Path]) -> Result<(),CompilationError> {
    println!("compiling {:?}", input_path.to_str());
    let assembly_filename = output_name.with_extension("asm");
    let object_filename = output_name.with_extension("o");
    let binary_filename = output_name;


    let tu = TranslationUnit::new(input_path)?;

    let mut formatter = TreeDisplayInfo::default();
    tu.display_ast(&mut formatter);
    println!("{}", formatter.get_text());
    
    println!("{}", tu.display_ir());

    tu.generate_assembly(&assembly_filename);

    //assemble
    let nasm_status = Command::new("nasm")
        .arg("-f elf64")
        .arg("-O0").arg("-g")
        .arg("-o")
        .arg(object_filename.clone())
        .arg(assembly_filename)
        .status();

    match nasm_status {
        Ok(status) if status.success() => {},
        _ => {
            return Err(CompilationError::ASMLINK("NASM failed to assemble the file".to_string()));
        }
    }

    //link
    let ld_status = Command::new("ld")
        .arg("-o")
        .arg(binary_filename)//link to the binary file name using:

        .arg("/usr/lib/x86_64-linux-gnu/crt1.o")//link c runtime
        .arg("/usr/lib/x86_64-linux-gnu/crti.o")//..
        .args(link_with)//link with other requested binaries

        .arg(object_filename)//link my code with the library
        .arg("-lc")//link with libc
        .arg("/usr/lib/x86_64-linux-gnu/crtn.o")//c runtime termination

        .arg("--dynamic-linker")
        .arg("/lib64/ld-linux-x86-64.so.2")//add a dynamic linker?
        
        .status();

    match ld_status {
        Ok(status) if status.success() => {},
        _ => {
            return Err(CompilationError::ASMLINK("Linker failed to link binary".to_string()));
        }
    }

    Ok(())
}