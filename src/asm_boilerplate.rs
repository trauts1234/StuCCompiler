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

        // ptr -> *
        // * -> ptr
        (DataType::POINTER(_), DataType::POINTER(_)) |
        (DataType::POINTER(_), DataType::RAW(BaseType::Scalar(ScalarType::Integer(_)))) => {
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


        (DataType::POINTER(_), DataType::POINTER(_)) |//pointer to pointer casts do nothing
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

        //pointer to integer conversion
        (DataType::POINTER(_), DataType::RAW(BaseType::Scalar(ScalarType::Integer(_)))) => {
            let mut asm = Assembly::make_empty();
            asm.add_instruction(AsmOperation::CAST { from_type: original.decay_to_primative(), to_type: new_type.decay_to_primative() });
            asm
        }
        
        

    }
}