use crate::{asm_generation::{asm_comment, asm_line, LogicalRegister, RegisterName}, data_type::{base_type::BaseType, data_type::DataType}, memory_size::MemoryLayout};
use std::fmt::Write;


pub fn global_var_label(var_name: &str) -> String {
    format!("GLOBVAR_{}", var_name)
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

pub fn mov_reg<T: RegisterName, U: RegisterName>(reg_size: &MemoryLayout, to: &T, from: &U) -> String {
    let to_name = to.generate_reg_name(reg_size);
    let from_name = from.generate_reg_name(reg_size);

    format!(
        "mov {}, {}", to_name, from_name
    )
}

pub fn cast_from_acc(original: &DataType, new_type: &DataType) -> String {

    assert!(!new_type.is_array());//cannot cast to array

    if new_type.underlying_type().is_va_arg() {
        assert!(new_type.get_modifiers().len() == 0);//can never have pointer to varadic arg
        return String::new();//cast to varadic arg does nothing, as types are not specified for va args
    }

    if original.is_array() {
        let ptr = original.decay();
        return cast_from_acc(&ptr, new_type);
    }

    let mut result = String::new();

    if original.is_pointer() {
        //cast pointer to u64
        //pointers are stored in memory just like u64, so no modifications needed
        let original_implicitly_as_u64 = DataType::new_from_base_type(&BaseType::U64, &Vec::new());
        //cast from
        return cast_from_acc(&original_implicitly_as_u64, new_type);
    }

    if new_type.is_pointer() {
        //cast u64 to pointer
        let new_implicitly_as_u64 = DataType::new_from_base_type(&BaseType::U64, &Vec::new());
        //cast from
        return cast_from_acc(original, &new_implicitly_as_u64);
    }

    if original.underlying_type().is_integer() && new_type.underlying_type().is_integer() {

        if new_type.underlying_type() == &BaseType::_BOOL {
            //boolean, so I need to cmp 0
            asm_line!(result, "cmp {}, 0", LogicalRegister::ACC.generate_reg_name(&original.memory_size()));
            //set to 1 or 0 based on whether that value was 0
            asm_line!(result, "setne {}", LogicalRegister::ACC.generate_reg_name(&MemoryLayout::from_bytes(1)));

            return result;
        }

        match (original.memory_size().size_bytes(), original.underlying_type().is_unsigned()) {
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

                let original_now_as_i64 = DataType::new_from_base_type(&BaseType::U64, &Vec::new());
                asm_line!(result, "{}", cast_from_acc(&original_now_as_i64, new_type));//cast the i64 back down to whatever new_type is
            }
            (x, true) => {
                let data_size = MemoryLayout::from_bytes(x);

                asm_comment!(result, "casting u{} to u64", data_size.size_bits());
                asm_line!(result, "{}", zero_extend_acc(&data_size));//zero extend rax to u64

                let original_now_as_u64 = DataType::new_from_base_type(&BaseType::U64, &Vec::new());
                asm_line!(result, "{}", cast_from_acc(&original_now_as_u64, new_type));//cast the u64 back down to whatever new_type is

            }
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