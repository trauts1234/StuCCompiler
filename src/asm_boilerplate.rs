use stack_management::stack_item::StackItemKey;

use crate::{asm_gen_data::AsmData, assembly::{assembly::Assembly, operand::{memory_operand::MemoryOperand, register::GPRegister, Operand, RegOrMem}, operation::AsmOperation}, data_type::{base_type::{BaseType, ScalarType}, recursive_data_type::DataType}};

pub fn cast_from_acc(original: &DataType, new_type: &DataType, asm_data: &AsmData) -> Assembly {
    match (original, new_type) {
        (_, DataType::ARRAY { .. }) => panic!("cannot cast to array"),

        (_, DataType::RAW(BaseType::VaArg)) => Assembly::make_empty(),//cast to varadic arg does nothing, as types are not specified for va args

        (DataType::ARRAY { .. }, _) => cast_from_acc(&original.decay(), new_type, asm_data),//decay arrays

        (_, _) => {
            let mut asm = Assembly::make_empty();
            asm.add_instruction(AsmOperation::CAST { from_type: original.decay_to_primative(), to_type: new_type.decay_to_primative() });
            asm
        }
    }
}

pub fn cast_from_memory(original: &StackItemKey, original_type: &DataType, new: &StackItemKey, new_type: &DataType, asm_data: &AsmData) -> Assembly {
    match (original_type, new_type) {
        // array -> *
        // * -> array
        (_, DataType::ARRAY { .. }) => panic!("cannot cast to array"),
        (DataType::ARRAY {..}, _) => cast_from_memory(original, &original_type.decay(), new, new_type, asm_data),//decay array to pointer and retry

        // structs
        (DataType::RAW(BaseType::Struct(x)), DataType::RAW(BaseType::Struct(y))) if x == y => {
            let mut result = Assembly::make_empty();
            result.add_instruction(AsmOperation::MEMCPY {
                size: original_type.memory_size(asm_data),
                from: MemoryOperand::SubFromBP(*original),
                to: MemoryOperand::SubFromBP(*new),
            });
            result
        }
        (DataType::RAW(BaseType::Struct(_)), DataType::RAW(BaseType::Struct(_))) => panic!("can't cast between different struct types"),
        (DataType::RAW(BaseType::Struct(_)), _) => panic!("can't cast struct to this type"),
        (_, DataType::RAW(BaseType::Struct(_))) => panic!("can't cast this to a struct"),

        //unions
        (DataType::RAW(BaseType::Union(x)), DataType::RAW(BaseType::Union(y))) if x == y => {
            let mut result = Assembly::make_empty();
            result.add_instruction(AsmOperation::MEMCPY {
                size: original_type.memory_size(asm_data),
                from: MemoryOperand::SubFromBP(*original),
                to: MemoryOperand::SubFromBP(*new),
            });
            result
        }
        (DataType::RAW(BaseType::Union(_)), DataType::RAW(BaseType::Union(_))) => panic!("can't cast between different union types"),
        (DataType::RAW(BaseType::Union(_)), _) => panic!("can't cast union to this type"),
        (_, DataType::RAW(BaseType::Union(_))) => panic!("can't cast this to a union"),

        // pointer and scalar casts
        (DataType::POINTER(_), DataType::POINTER(_)) |
        (DataType::RAW(BaseType::Scalar(_)), DataType::RAW(BaseType::Scalar(_))) |
        (DataType::POINTER(_), DataType::RAW(BaseType::Scalar(ScalarType::Integer(_)))) |
        (DataType::RAW(BaseType::Scalar(ScalarType::Integer(_))), DataType::POINTER(_)) => {
            let mut asm = Assembly::make_empty();
            //put in acc
            asm.add_instruction(AsmOperation::MOV {
                to: RegOrMem::GPReg(GPRegister::acc()),
                from: Operand::Mem(MemoryOperand::SubFromBP(*original)),
                size: original_type.memory_size(asm_data),
            });
            //cast
            asm.add_instruction(AsmOperation::CAST { from_type: original_type.decay_to_primative(), to_type: new_type.decay_to_primative() });
            //put in new location
            asm.add_instruction(AsmOperation::MOV {
                to: RegOrMem::Mem(MemoryOperand::SubFromBP(*new)),
                from: Operand::GPReg(GPRegister::acc()),
                size: new_type.memory_size(asm_data),
            });
            asm
        }
        (DataType::POINTER(_), _) => panic!("cannot convert a pointer to this type"),
        (_, DataType::POINTER(_)) => panic!("cannot convert this type to a pointer"),

        // void -> *
        // * -> void
        (DataType::RAW(BaseType::VOID), _) |
        (_, DataType::RAW(BaseType::VOID)) => panic!("can't convert to and from the void type"),

        // varadic -> *
        // * -> varadic
        (_, DataType::RAW(BaseType::VaArg)) => //anything to type-less varadic does nothing
        {
            let mut result = Assembly::make_empty();
            //as the result is varadic with no type, just copy the data
            result.add_instruction(AsmOperation::MEMCPY {
                size: original_type.memory_size(asm_data),
                from: MemoryOperand::SubFromBP(*original),
                to: MemoryOperand::SubFromBP(*new),
            });
            result
        }
        (DataType::RAW(BaseType::VaArg), _) => panic!("not sure how to convert a varadic arg to this type"), 

    }
}