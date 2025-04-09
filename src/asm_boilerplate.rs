use crate::{asm_gen_data::AsmData, assembly::{assembly::Assembly, operand::{Operand, AsmRegister}, operation::{AsmComparison, AsmOperation}}, data_type::{base_type::BaseType, recursive_data_type::DataType}, memory_size::MemoryLayout};

pub fn cast_from_acc(original: &DataType, new_type: &DataType, asm_data: &AsmData) -> Assembly {
    match (original, new_type) {
        (_, DataType::ARRAY { size:_, element:_ }) => panic!("cannot cast to array"),
        (_, DataType::RAW(BaseType::VaArg)) => Assembly::make_empty(),//cast to varadic arg does nothing, as types are not specified for va args
        (DataType::ARRAY { size:_, element:_ }, _) => cast_from_acc(&original.decay(), new_type, asm_data),//decay arrays
        (DataType::POINTER(_), _) => cast_from_acc(&DataType::RAW(BaseType::U64), new_type, asm_data),//pointers are stored in memory just like u64, so cast to u64
        (_, DataType::POINTER(_)) => cast_from_acc(original, &DataType::RAW(BaseType::U64), asm_data),// ''
        (DataType::RAW(from_raw), DataType::RAW(to_raw)) => {
            //TODO move to separate function
            let mut result = Assembly::make_empty();
            if to_raw == &BaseType::_BOOL {
                //boolean, so I need to cmp 0
                result.add_instruction(AsmOperation::CMP { 
                    lhs: Operand::Register(AsmRegister::acc()),
                    rhs: Operand::ImmediateValue("0".to_string()),
                    data_type: DataType::RAW(from_raw.clone())
                });
                //set to 1 or 0 based on whether that value was 0
                result.add_instruction(AsmOperation::SETCC {
                    destination: Operand::Register(AsmRegister::acc()),
                    comparison: AsmComparison::NE,
                });
    
                return result;
            }

            if to_raw.is_integer() && from_raw.is_integer() && to_raw.memory_size(asm_data) <= from_raw.memory_size(asm_data) {
                return Assembly::make_empty();//casting integer to smaller integer, no change required
            }
    
            match (from_raw.memory_size(asm_data).size_bytes(), from_raw.is_unsigned()) {
                (x, false) => {
                    let data_size = MemoryLayout::from_bytes(x);

                    result.add_commented_instruction(AsmOperation::BLANK, format!("casting i{} to i64", data_size.size_bits()));

                    result.add_instruction(AsmOperation::SignExtendACC { old_size: data_size});//sign extend rax to i64

                    result.merge(&cast_from_acc(&DataType::RAW(BaseType::I64), new_type, asm_data));//cast the i64 back down to whatever new_type is
                }
                (x, true) => {
                    let data_size = MemoryLayout::from_bytes(x);

                    result.add_commented_instruction(AsmOperation::BLANK, format!("casting u{} to u64", data_size.size_bits()));
                    result.add_instruction(AsmOperation::ZeroExtendACC { old_size: data_size});//zero extend rax to u64

                    result.merge(&cast_from_acc(&DataType::RAW(BaseType::U64), new_type, asm_data));//cast the u64 back down to whatever new_type is
    
                }
            }

            result
        }
    }
}

