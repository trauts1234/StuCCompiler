use crate::{asm_boilerplate, asm_generation::{asm_comment, asm_line, LogicalRegister, RegisterName}, memory_size::MemoryLayout, type_info::{DataType, TypeInfo}};
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
"
global main

SECTION .note.GNU-stack ;disable executing the stack
SECTION .text
{}",instructions)

}

pub fn pop_reg<T: RegisterName>(reg_size: &MemoryLayout, reg_type: &T) -> String {
    let reg_name = reg_type.generate_reg_name(reg_size);

    format!(
        ";pop {}\nmov {}, [rsp]\nadd rsp, {}",reg_name, reg_name, reg_size.size_bytes()
    )
}

pub fn push_reg<T: RegisterName>(reg_size: &MemoryLayout, reg_type: &T) -> String {
    let reg_name = reg_type.generate_reg_name(reg_size);

    format!(
        ";push {}\nsub rsp, {}\nmov [rsp], {}", reg_name, reg_size.size_bytes(), reg_name
    )
}

pub fn cast_from_stack(original: &DataType, new_type: &DataType) -> String {

    if let Some(ptr) = original.decay_array_to_pointer() {
        //arrays are just pointers in disguise
        return cast_from_stack(&ptr, new_type);
    }

    let mut result = String::new();

    if original.is_pointer() {
        //cast pointer to u64
        //pointers are stored in memory just like u64, so no modifications needed
        let original_implicitly_as_u64 = DataType {
            type_info: vec![TypeInfo::UNSIGNED, TypeInfo::LONG, TypeInfo::LONG, TypeInfo::INT],
            modifiers: Vec::new(),
        };
        //cast from
        return cast_from_stack(&original_implicitly_as_u64, new_type);
    }

    if original.underlying_type_is_integer() && new_type.underlying_type_is_integer() {
        match (original.memory_size().size_bytes(), original.underlying_type_is_unsigned()) {
            (8, _) => {

                if new_type.memory_size().size_bytes() == 8{
                    return String::new();//we would just be popping a register just to put it back again??? return here
                }

                asm_comment!(result, "casting 64 bit integer to {} bit integer", new_type.memory_size().size_bits());

                asm_line!(result, "{}", asm_boilerplate::pop_reg(&MemoryLayout::from_bytes(8), &LogicalRegister::ACC));//grab the unsigned 64 bit number

                //no matter the signedness of original, you just need to get the bottom few bits of it,
                //because positive numbers are the same for uxx and ixx in original
                //negative numbers are already sign extended, so need no special treatment
                asm_line!(result, "{}", asm_boilerplate::push_reg(&new_type.memory_size(), &LogicalRegister::ACC));
            }
            (4, false) => {
                asm_comment!(result, "casting i32 to i64");
                asm_line!(result, "{}", asm_boilerplate::pop_reg(&MemoryLayout::from_bytes(4), &LogicalRegister::ACC));//32 bit input -> eax
                asm_line!(result, "movsxd rax, eax");//sign extend eax -> rax
                asm_line!(result, "{}", asm_boilerplate::push_reg(&MemoryLayout::from_bytes(8), &LogicalRegister::ACC));//rax -> 64 bit output

                let original_now_as_i64 = DataType {
                    type_info:vec![TypeInfo::LONG, TypeInfo::LONG, TypeInfo::INT],
                    modifiers: Vec::new(),
                };
                asm_line!(result, "{}", cast_from_stack(&original_now_as_i64, new_type));//cast the i64 back down to whatever new_type is
            },
            (size, unsigned) => panic!("casting this type of integer is not implemented: {} bytes, unsigned?: {}", size, unsigned)
        }
        return result;
    }


    panic!("can't cast these");
}

/**
 * extend eax to edx:eax (64 bit register pair)
 * divide edx:eax by ecx
 */
pub const I32_DIVIDE_AX_BY_CX: &str =
"cdq
idiv ecx";

pub const I64_DIVIDE_AX_BY_CX: &str =
"cqo
idiv rcx";
