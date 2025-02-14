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
    call func_main
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

pub const I32_ADD: &str =
";add two i32s
pop rax
pop rbx
add eax, ebx
movsxd rax, eax
push rax";

/**
 * read the two numbers
 * multiply them to edx:eax (64 bit register pair)
 * sign extend the bottom 32 bits to 64 bits
 * then push to stack
 */
pub const I32_MULTIPLY: &str =
";multiply two i32s
pop rax
pop rbx
imul eax, ebx
movsxd rax, eax
push rax";

/**
 * read the denominator to rbx
 * read the numerator to rax
 * extend rax to rdx:rax (128 bit register pair)
 * divide rdx:rax by rbx
 * sign extend the result, then push to stack
 */
pub const I32_DIVIDE: &str =
";divide two i32s
pop rbx
pop rax
cdq
idiv ebx
movsxd rax, eax
push rax";
