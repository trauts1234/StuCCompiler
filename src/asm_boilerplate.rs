use crate::{asm_boilerplate, asm_generation::{self, asm_comment, asm_line}, type_info::{DataType, TypeInfo}};
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

pub fn pop_reg(reg_name: &str) -> String {
    let prefix = reg_name.chars().next().unwrap();

    let bytes_sub_from_sp = match prefix {
        'e' => 4,
        'r' => 8,
        _ => panic!("undefined register prefix {}", prefix)
    };

    format!(
        ";pop {}\nmov {}, [rsp]\nadd rsp, {}",reg_name, reg_name, bytes_sub_from_sp
    )
}

pub fn push_reg(reg_name: &str) -> String {
    let prefix = reg_name.chars().next().unwrap();

    let bytes_sub_from_sp = match prefix {
        'e' => 4,
        'r' => 8,
        _ => panic!("undefined register prefix {}", prefix)
    };

    format!(
        ";push {}\nsub rsp, {}\nmov [rsp], {}", reg_name, bytes_sub_from_sp, reg_name
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
                asm_comment!(result, "casting 64 bit integer to {} bit integer", new_type.memory_size().size_bytes());
                asm_line!(result, "{}", asm_boilerplate::pop_reg("rax"));//grab the unsigned 64 bit number

                let resultant_reg_name = asm_generation::generate_reg_name(&new_type.memory_size(), "ax");//which type of ax register will the value be in

                //no matter the signedness of original, you just need to get the bottom few bits of it,
                //because positive numbers are the same for uxx and ixx in original
                //negative numbers are already sign extended, so need no special treatment
                asm_line!(result, "{}", asm_boilerplate::push_reg(&resultant_reg_name));
            }
            (4, false) => {
                asm_comment!(result, "casting i32 to i64");
                asm_line!(result, "{}", asm_boilerplate::pop_reg("eax"));//32 bit input -> eax
                asm_line!(result, "movsxd rax, eax");//sign extend eax -> rax
                asm_line!(result, "{}", asm_boilerplate::push_reg("rax"));//rax -> 64 bit output

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
 * divide edx:eax by ebx
 */
pub const I32_DIVIDE: &str =
"cdq
idiv ebx";
