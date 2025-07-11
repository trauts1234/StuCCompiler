use crate::{asm_gen_data::AsmData, assembly::{assembly::Assembly, comparison::AsmComparison, operand::{immediate::ImmediateValue, register::{GPRegister, MMRegister}, Operand, RegOrMem}, operation::AsmOperation}, data_type::{base_type::{BaseType, IntegerType, ScalarType}, recursive_data_type::DataType}};
use memory_size::MemorySize;

pub fn cast_from_acc(original: &DataType, new_type: &DataType, asm_data: &AsmData) -> Assembly {
    match (original, new_type) {
        (_, DataType::UNKNOWNSIZEARRAY { .. }) |
        (_, DataType::ARRAY { .. }) => panic!("cannot cast to array"),

        (_, DataType::RAW(BaseType::VaArg)) => Assembly::make_empty(),//cast to varadic arg does nothing, as types are not specified for va args

        (DataType::UNKNOWNSIZEARRAY { .. }, _) |
        (DataType::ARRAY { .. }, _) => cast_from_acc(&original.decay(), new_type, asm_data),//decay arrays

        (_, _) => cast_raw_from_acc(&original.decay_to_primative(), &new_type.decay_to_primative(), asm_data)
    }
}

pub fn cast_raw_from_acc(from_raw: &ScalarType, to_raw: &ScalarType, asm_data: &AsmData) -> Assembly {
    let mut result = Assembly::make_empty();
    if to_raw == &ScalarType::Integer(IntegerType::_BOOL) {
        //boolean, so I need to cmp 0
        result.add_instruction(AsmOperation::CMP { 
            lhs: Operand::GPReg(GPRegister::acc()),
            rhs: Operand::Imm(ImmediateValue("0".to_string())),
            data_type: from_raw.clone()
        });
        //set to 1 or 0 based on whether that value was 0
        result.add_instruction(AsmOperation::SETCC {
            comparison: AsmComparison::NE,
        });

        return result;
    }

    match (from_raw, to_raw) {
        (ScalarType::Integer(from_int), ScalarType::Integer(to_int)) => {
            if to_int.memory_size() <= from_int.memory_size() {
                return Assembly::make_empty();//casting integer to smaller integer, no change required
            }

            match (from_int.memory_size().size_bytes(), from_int.is_unsigned()) {
                (x, false) => {
                    let data_size = MemorySize::from_bytes(x);

                    result.add_comment(format!("casting signed {} integer to i64", data_size));

                    result.add_instruction(AsmOperation::SignExtendACC { old_size: data_size});//sign extend rax to i64

                    //implicit truncate to new type
                }
                (x, true) => {
                    let data_size = MemorySize::from_bytes(x);

                    result.add_commented_instruction(AsmOperation::BLANK, format!("casting unsigned {} integer to u64", data_size));
                    result.add_instruction(AsmOperation::ZeroExtendACC { old_size: data_size});//zero extend rax to u64

                    //implicit truncate to new type

                }
            }
        },

        //integer to float
        (ScalarType::Integer(from_int), ScalarType::Float(to_float)) => {
            if from_int.is_unsigned() {
                cast_raw_from_acc(from_raw, &ScalarType::Integer(IntegerType::U64), asm_data);//cast u__ to u64
            } else {
                cast_raw_from_acc(from_raw, &ScalarType::Integer(IntegerType::I64), asm_data);//cast i__ to i64
            };

            //cast the number to float/double
            result.add_commented_instruction(AsmOperation::GP64CastMMX { from: GPRegister::acc(), to: MMRegister::acc(), from_is_signed: !from_int.is_unsigned(), to_type: to_float.clone() }, format!("casting GP accumulator to FP accumulator"));
        },

        //float to float cast
        (ScalarType::Float(from_float), ScalarType::Float(to_float)) => {
            result.add_instruction(AsmOperation::MMXCastMMX { from: MMRegister::acc(), to: MMRegister::acc(), from_type: from_float.clone(), to_type: to_float.clone() });
        }

        _ => panic!()
    }

    result
}

