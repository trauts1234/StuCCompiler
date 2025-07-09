use crate::{asm_gen_data::AsmData, assembly::{assembly::Assembly, comparison::AsmComparison, operand::{immediate::ImmediateValue, register::{GPRegister, MMRegister}, Operand, RegOrMem}, operation::AsmOperation}, data_type::{base_type::BaseType, recursive_data_type::DataType}};
use memory_size::MemorySize;

pub fn cast_from_acc(original: &DataType, new_type: &DataType, asm_data: &AsmData) -> Assembly {
    match (original, new_type) {
        (_, DataType::UNKNOWNSIZEARRAY { .. }) |
        (_, DataType::ARRAY { .. }) => panic!("cannot cast to array"),

        (_, DataType::RAW(BaseType::VaArg)) => Assembly::make_empty(),//cast to varadic arg does nothing, as types are not specified for va args

        (DataType::UNKNOWNSIZEARRAY { .. }, _) |
        (DataType::ARRAY { .. }, _) => cast_from_acc(&original.decay(), new_type, asm_data),//decay arrays
        
        (DataType::POINTER(_), _) => cast_from_acc(&DataType::RAW(BaseType::U64), new_type, asm_data),//pointers are stored in memory just like u64, so cast to u64
        (_, DataType::POINTER(_)) => cast_from_acc(original, &DataType::RAW(BaseType::U64), asm_data),// ''
        (DataType::RAW(from_raw), DataType::RAW(to_raw)) => cast_raw_from_acc(from_raw, to_raw, asm_data)
    }
}

fn cast_raw_from_acc(from_raw: &BaseType, to_raw: &BaseType, asm_data: &AsmData) -> Assembly {
    let mut result = Assembly::make_empty();
    println!("from {:?} to {:?}", from_raw, to_raw);
    if to_raw == &BaseType::_BOOL {
        //boolean, so I need to cmp 0
        result.add_instruction(AsmOperation::CMP { 
            lhs: Operand::Reg(GPRegister::acc()),
            rhs: Operand::Imm(ImmediateValue("0".to_string())),
            data_type: DataType::RAW(from_raw.clone())
        });
        //set to 1 or 0 based on whether that value was 0
        result.add_instruction(AsmOperation::SETCC {
            destination: RegOrMem::GPReg(GPRegister::acc()),
            comparison: AsmComparison::NE,
        });

        return result;
    }

    match (from_raw.is_integer(), to_raw.is_integer()) {
        (true, true) => {
            if to_raw.memory_size(asm_data) <= from_raw.memory_size(asm_data) {
                return Assembly::make_empty();//casting integer to smaller integer, no change required
            }

            match (from_raw.memory_size(asm_data).size_bytes(), from_raw.is_unsigned()) {
                (x, false) => {
                    let data_size = MemorySize::from_bytes(x);

                    result.add_comment(format!("casting signed {} integer to i64", data_size));

                    result.add_instruction(AsmOperation::SignExtendACC { old_size: data_size});//sign extend rax to i64

                    result.merge(&cast_raw_from_acc(&BaseType::I64, to_raw, asm_data));//cast the i64 back down to whatever new_type is
                }
                (x, true) => {
                    let data_size = MemorySize::from_bytes(x);

                    result.add_commented_instruction(AsmOperation::BLANK, format!("casting unsigned {} integer to u64", data_size));
                    result.add_instruction(AsmOperation::ZeroExtendACC { old_size: data_size});//zero extend rax to u64

                    result.merge(&cast_raw_from_acc(&BaseType::U64, to_raw, asm_data));//cast the u64 back down to whatever new_type is

                }
            }
        },

        //integer to float
        (true, false) => {
            let casted_from = if from_raw.is_signed() {
                cast_raw_from_acc(from_raw, &BaseType::I64, asm_data);//cast i__ to i64
                BaseType::I64
            } else {
                cast_raw_from_acc(from_raw, &BaseType::U64, asm_data);//cast u__ to u64
                BaseType::U64
            };

            match (casted_from, to_raw) {
                (BaseType::I64, BaseType::F32) => {
                    //cast the number to
                    result.add_commented_instruction(AsmOperation::I64ToF32 { from: RegOrMem::GPReg(GPRegister::acc()), to: MMRegister::acc() }, format!("casting GP accumulator to FP accumulator"));
                }
                _ => panic!("invalid type for float")
            }
        },

        //float to float cast
        (false, false) => {
            match (from_raw.memory_size(asm_data), to_raw.memory_size(asm_data)) {
                (x, y) if x == y => 
                    return Assembly::make_empty(),//same size floats, no conversion needed

                _ => panic!()
            }
        }

        _ => panic!()
    }

    result
}

