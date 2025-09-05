//! This file figures out what type each arg is

use memory_size::MemorySize;

use crate::{asm_gen_data::GetStructUnion, data_type::{base_type::{BaseType, ScalarType}, recursive_data_type::DataType}, declaration::Declaration};

#[derive(PartialEq, Clone)]
pub enum StructEightbytePreferredLocation {
    InGP,
    InMMX,
    //if it were preferred in memory, the whole struct should be in memory
}

#[derive(PartialEq, Clone)]
pub enum PreferredParamLocation {
    InGP,
    InMemory,
    InMMX,
    Struct {l: StructEightbytePreferredLocation, r: StructEightbytePreferredLocation},
}

impl PreferredParamLocation {

    /// Calculates where a param *should* be stored, assuming enough registers are present
    /// 
    /// Do not pass VOID, as it will `panic!`
    pub fn param_from_type(data_type: &DataType, get_struct_union: &dyn GetStructUnion) -> PreferredParamLocation {
        match data_type {
            DataType::UNKNOWNSIZEARRAY { .. } |
            DataType::ARRAY {..} => PreferredParamLocation::InGP,//decays to a pointer, which is integer

            DataType::POINTER(_) => PreferredParamLocation::InGP,// pointer can be passed as an integer type
            DataType::RAW(base_type) => 
                Self::param_from_base_type(base_type, get_struct_union),
        }
    }

    fn param_from_base_type(base_type: &BaseType, get_struct_union: &dyn GetStructUnion) -> PreferredParamLocation {
        match base_type {
            BaseType::Struct(struct_name) => {
                let struct_type = get_struct_union.get_struct(&struct_name);

                match struct_type.calculate_size().unwrap().size_bytes() {
                    ..=16 => {
                        let args_iter = struct_type.get_all_members().as_ref().expect("tried to pass a struct as a param but it had no members").iter();
                        
                        let is_first_eightbyte_predicate = |(decl, offset): &&(Declaration, MemorySize)| {
                            let last_byte_of_member_offset = decl.data_type.memory_size(get_struct_union) + *offset;

                            last_byte_of_member_offset.size_bytes() <= 8
                        };

                        let first_eightbyte_types: Vec<_> = args_iter.clone()
                            .take_while(is_first_eightbyte_predicate)
                            .map(|(decl, _)| Self::param_from_type(&decl.data_type, get_struct_union))
                            .collect();
                        let second_eightbyte_types: Vec<_> = args_iter
                            .skip_while(is_first_eightbyte_predicate)
                            .map(|(decl, _)| Self::param_from_type(&decl.data_type, get_struct_union))
                            .collect();

                        let first_eightbyte = classify_eightbyte(&first_eightbyte_types).unwrap();
                        let second_eightbyte = classify_eightbyte(&second_eightbyte_types);

                        match (first_eightbyte, second_eightbyte) {
                            (PreferredParamLocation::InMemory, _) |
                            (_, Some(PreferredParamLocation::InMemory)) => PreferredParamLocation::InMemory,//if either must go in memory, whole thing needs to go in memory

                            (x, None) => x,//struct is too small, so just need one arg type to store it

                            (x, Some(y)) => Self::Struct { l:x.try_into().unwrap(), r: y.try_into().unwrap() }
                        }
                    },
                    17.. => PreferredParamLocation::InMemory//too big
                }
            },
            BaseType::Scalar(ScalarType::Integer(x)) => {
                assert!(x.memory_size().size_bytes() <= 8);//must be able to fit in a register
                PreferredParamLocation::InGP
            },
            BaseType::Scalar(ScalarType::Float(_)) => {
                PreferredParamLocation::InMMX
            }
            x => panic!("tried to calculate param type from unknown type: {:?}", x)
        }
    }
}

impl TryInto<StructEightbytePreferredLocation> for PreferredParamLocation {
    type Error = ();

    fn try_into(self) -> Result<StructEightbytePreferredLocation, Self::Error> {
        match self {
            Self::InGP => Ok(StructEightbytePreferredLocation::InGP),
            Self::InMMX => Ok(StructEightbytePreferredLocation::InMMX),
            _ => Err(())
        }
    }
}

fn classify_eightbyte(member_types: &[PreferredParamLocation]) -> Option<PreferredParamLocation> {
    member_types
    .iter()
    .fold(None, |curr, x| Some(classify_pair(curr, x)))
}
fn classify_pair(first: Option<PreferredParamLocation>, second: &PreferredParamLocation) -> PreferredParamLocation {
    match (&first, second) {
        (Some(x), y) if x == y => y.clone(),//if pair are the same type, result is the same type

        (None, y) => y.clone(),//if either is NO_CLASS, return the other

        (Some(PreferredParamLocation::InMemory), _) |
        (_, PreferredParamLocation::InMemory) => PreferredParamLocation::InMemory,//if either is MEMORY, result is MEMORY

        (Some(PreferredParamLocation::InGP), _) |
        (_, PreferredParamLocation::InGP) => PreferredParamLocation::InGP,//if either is INTEGER, result is INTEGER

        _ => PreferredParamLocation::InMMX//apparently this is the default?
    }
}