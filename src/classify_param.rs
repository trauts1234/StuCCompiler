use crate::{asm_gen_data::AsmData, data_type::{base_type::BaseType, recursive_data_type::RecursiveDataType}, declaration::Declaration, memory_size::MemoryLayout};


#[derive(PartialEq, Clone)]
pub enum ArgType {
    INTEGER,
    MEMORY,
    NoClass,//null type
    STRUCT {first_eightbyte: Box<ArgType>, second_eightbyte: Box<ArgType>}
}

impl ArgType {

    pub fn param_from_type(data_type: &RecursiveDataType, asm_data: &AsmData) -> ArgType {
        match data_type {
            RecursiveDataType::ARRAY {..} => ArgType::INTEGER,//decays to a pointer, which is integer
            RecursiveDataType::POINTER(_) => ArgType::INTEGER,// ''
            RecursiveDataType::RAW(base_type) => 
                Self::param_from_base_type(base_type, asm_data),
        }
    }

    fn param_from_base_type(base_type: &BaseType, asm_data: &AsmData) -> ArgType {
        match base_type {
            BaseType::STRUCT(struct_name) => {
                let struct_type = asm_data.get_struct(&struct_name);

                match struct_type.calculate_size().unwrap().size_bytes() {
                    ..=16 => {
                        let args_iter = struct_type.get_all_members().as_ref().expect("tried to pass a struct as a param but it had no members").iter();
                        
                        let is_first_eightbyte_predicate = |(decl, offset): &&(Declaration, MemoryLayout)| {
                            let last_byte_of_member_offset = decl.get_type().memory_size(asm_data) + *offset;

                            last_byte_of_member_offset.size_bytes() <= 8
                        };

                        let first_eightbyte_types: Vec<_> = args_iter.clone()
                            .take_while(is_first_eightbyte_predicate)
                            .map(|(decl, _)| Self::param_from_type(decl.get_type(), asm_data))
                            .collect();
                        let second_eightbyte_types: Vec<_> = args_iter
                            .skip_while(is_first_eightbyte_predicate)
                            .map(|(decl, _)| Self::param_from_type(decl.get_type(), asm_data))
                            .collect();

                        let first_eightbyte = classify_eightbyte(&first_eightbyte_types);
                        let second_eightbyte = classify_eightbyte(&second_eightbyte_types);

                        match (first_eightbyte, second_eightbyte) {
                            (ArgType::MEMORY, _) |
                            (_, ArgType::MEMORY) => ArgType::MEMORY,

                            (x, ArgType::NoClass) => x,//struct is too small, so just need one arg type to store it

                            (x, y) => ArgType::STRUCT { first_eightbyte: Box::new(x), second_eightbyte: Box::new(y) }
                        }
                    },
                    17.. => ArgType::MEMORY//too big
                }
            },
            x if x.is_integer() => {
                assert!(x.memory_size(asm_data).size_bits() <= 64);//must be able to fit in a register
                ArgType::INTEGER
            },
            x => panic!("tried to calculate param type from unknown type: {:?}", x)
        }
    }
}

fn classify_eightbyte(member_types: &[ArgType]) -> ArgType {
    member_types
    .iter()
    .fold(ArgType::NoClass, classify_pair)
}
fn classify_pair(first: ArgType, second: &ArgType) -> ArgType {
    match (&first, second) {
        (x, y) if x == y => y.clone(),//if pair are the same type, result is the same type

        (ArgType::NoClass, y) => y.clone(),//if either is NO_CLASS, return the other
        (x, ArgType::NoClass) => x.clone(),// ''

        (ArgType::MEMORY, _) |
        (_, ArgType::MEMORY) => ArgType::MEMORY,//if either is MEMORY, result is MEMORY

        (ArgType::INTEGER, _) |
        (_, ArgType::INTEGER) => ArgType::INTEGER,//if either is INTEGER, result is INTEGER

        _ => panic!("unknown pattern when classifying struct type")
    }
}