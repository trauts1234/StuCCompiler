use crate::{asm_gen_data::AsmData, asm_generation::{asm_comment, asm_line, LogicalRegister, AssemblyOperand}, data_type::{base_type::BaseType, recursive_data_type::RecursiveDataType}, memory_size::MemoryLayout};
use std::fmt::Write;


pub fn global_var_label(var_name: &str) -> String {
    format!("GLOBVAR_{}", var_name)
}

pub fn mov_asm<T: AssemblyOperand, U: AssemblyOperand>(reg_size: MemoryLayout, to: &T, from: &U) -> String {
    let to_name = to.generate_name(reg_size);
    let from_name = from.generate_name(reg_size);

    format!(
        "mov {}, {}", to_name, from_name
    )
}

pub fn cast_from_acc(original: &RecursiveDataType, new_type: &RecursiveDataType, asm_data: &AsmData) -> String {
    match (original, new_type) {
        (_, RecursiveDataType::ARRAY { size:_, element:_ }) => panic!("cannot cast to array"),
        (_, RecursiveDataType::RAW(BaseType::VaArg)) => String::new(),//cast to varadic arg does nothing, as types are not specified for va args
        (RecursiveDataType::ARRAY { size:_, element:_ }, _) => cast_from_acc(&original.decay(), new_type, asm_data),//decay arrays
        (RecursiveDataType::POINTER(_), _) => cast_from_acc(&RecursiveDataType::RAW(BaseType::U64), new_type, asm_data),//pointers are stored in memory just like u64, so cast to u64
        (_, RecursiveDataType::POINTER(_)) => cast_from_acc(original, &RecursiveDataType::RAW(BaseType::U64), asm_data),// ''
        (RecursiveDataType::RAW(from_raw), RecursiveDataType::RAW(to_raw)) => {
            //TODO move to separate function
            let mut result = String::new();
            if to_raw == &BaseType::_BOOL {
                //boolean, so I need to cmp 0
                asm_line!(result, "cmp {}, 0", LogicalRegister::ACC.generate_name(from_raw.memory_size(asm_data)));
                //set to 1 or 0 based on whether that value was 0
                asm_line!(result, "setne {}", LogicalRegister::ACC.generate_name(MemoryLayout::from_bytes(1)));
    
                return result;
            }
    
            match (from_raw.memory_size(asm_data).size_bytes(), from_raw.is_unsigned()) {
                (8, _) => {
                    //no matter the signedness of original, you just need to get the bottom few bits of it,
                    //because positive numbers are the same for uxx and ixx in original
                    //negative numbers are already sign extended, so need no special treatment
    
                    //do nothing
                }
                (x, false) => {
                    let data_size = MemoryLayout::from_bytes(x);
    
                    asm_comment!(result, "casting i{} to i64", data_size.size_bits());
                    asm_line!(result, "{}", sign_extend_acc(&data_size));//sign extend rax to i64
    
                    asm_line!(result, "{}", cast_from_acc(&RecursiveDataType::RAW(BaseType::I64), new_type, asm_data));//cast the i64 back down to whatever new_type is
                }
                (x, true) => {
                    let data_size = MemoryLayout::from_bytes(x);
    
                    asm_comment!(result, "casting u{} to u64", data_size.size_bits());
                    asm_line!(result, "{}", zero_extend_acc(&data_size));//zero extend rax to u64

                    asm_line!(result, "{}", cast_from_acc(&RecursiveDataType::RAW(BaseType::I64), new_type, asm_data));//cast the u64 back down to whatever new_type is
    
                }
            }

            result
        }
    }
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

fn sign_extend_acc(original: &MemoryLayout) -> String {
    match original.size_bits() {
        8 => format!("cbw\n{}", sign_extend_acc(&MemoryLayout::from_bits(16))),
        16 => format!("cwde\n{}", sign_extend_acc(&MemoryLayout::from_bits(32))),
        32 => format!("cdqe\n"),
        _ => panic!("tried to sign extend unknown size")
    }
}

fn zero_extend_acc(original: &MemoryLayout) -> String {
    match original.size_bits() {
        8 => String::from("movzx rax, al\n"),
        16 => String::from("movzx rax, ax\n"),
        32 => String::new(), // Writing to EAX automatically zeroes RAX's upper half.
        _ => panic!("tried to zero extend unknown size")
    }
}