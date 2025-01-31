use std::fmt::Write;

pub fn add_boilerplate(instructions: String) -> String {
    /*
    * set up some boilerplate, including:
    * global the _start label so that the linker has a main function to use
    * start the .text section for instructions
    * define _start:
    * * run the main program
    * * set up exit syscall with return code grabbed from eax(assuming that main returns int)
    */
    format!(
"global _start

SECTION .text
_start:
    call main
    mov edi, eax
    mov rax, 60
    syscall

{}",instructions)

}

pub fn func_exit_boilerplate() -> String {
    let mut result = String::new();
    //remove stack frame
    writeln!(result, "mov rsp, rbp").unwrap();
    writeln!(result, "pop rbp").unwrap();
    writeln!(result, "ret").unwrap();

    result
}