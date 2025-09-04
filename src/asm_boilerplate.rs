use crate::{asm_gen_data::AsmData, assembly::{assembly::Assembly, operation::AsmOperation}, data_type::{base_type::BaseType, recursive_data_type::DataType}};

pub fn cast_from_acc(original: &DataType, new_type: &DataType, asm_data: &AsmData) -> Assembly {
    match (original, new_type) {
        (_, DataType::UNKNOWNSIZEARRAY { .. }) |
        (_, DataType::ARRAY { .. }) => panic!("cannot cast to array"),

        (_, DataType::RAW(BaseType::VaArg)) => Assembly::make_empty(),//cast to varadic arg does nothing, as types are not specified for va args

        (DataType::UNKNOWNSIZEARRAY { .. }, _) |
        (DataType::ARRAY { .. }, _) => cast_from_acc(&original.decay(), new_type, asm_data),//decay arrays

        (_, _) => {
            let mut asm = Assembly::make_empty();
            println!("cast {:?} to {:?}", original, new_type);
            asm.add_instruction(AsmOperation::CAST { from_type: original.decay_to_primative(), to_type: new_type.decay_to_primative() });
            asm
        }
    }
}