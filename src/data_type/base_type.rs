use std::fmt::Display;

use crate::{asm_gen_data::GetStruct, data_type::type_token::TypeInfo, struct_definition::StructIdentifier};
use memory_size::MemorySize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaseType {
    VOID,
    VaArg,//varadic arg has a special type as it has no type?
    _BOOL,
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
    STRUCT(StructIdentifier)
}

impl BaseType {
    pub fn is_void(&self) -> bool {
        self == &Self::VOID
    }
    pub fn is_va_arg(&self) -> bool {
        self == &Self::VaArg
    }
    pub fn is_integer(&self) -> bool {
        match self {
            BaseType::VOID | BaseType::VaArg | BaseType::STRUCT(_) |
            BaseType::F32 | BaseType::F64 => false,

            BaseType::_BOOL |
            BaseType::I8 | 
            BaseType::U8 | 
            BaseType::I16 | 
            BaseType::U16 | 
            BaseType::I32 | 
            BaseType::U32 | 
            BaseType::I64 | 
            BaseType::U64 => true,
        }
    }
    
    pub fn is_unsigned(&self) -> bool {
        match self {
            BaseType::VOID | BaseType::VaArg | BaseType::STRUCT(_) => panic!("tried to detect signedness of void or varadic arg"),

            BaseType::I8 | 
            BaseType::I16 | 
            BaseType::I32 | 
            BaseType::I64 |
            BaseType::F32 |
            BaseType::F64 => false,

            BaseType::_BOOL |
            BaseType::U8 | 
            BaseType::U16 | 
            BaseType::U32 | 
            BaseType::U64 => true,
        }
    }
    pub fn is_signed(&self) -> bool {
        !self.is_unsigned()
    }

    pub fn memory_size(&self, struct_info: &dyn GetStruct) -> MemorySize {
        match self {
            BaseType::VOID => panic!("tried to get size of void"),
            BaseType::VaArg => panic!("tried to get size of varadic arg"),

            BaseType::STRUCT(x) => struct_info.get_struct(x).calculate_size().expect("tried to calculate size of partially declared struct"),

            BaseType::_BOOL |
            BaseType::I8 |
            BaseType::U8 => MemorySize::from_bytes(1),

            BaseType::I16 |
            BaseType::U16 => MemorySize::from_bytes(2),

            BaseType::I32 |
            BaseType::U32 |
            BaseType::F32 => MemorySize::from_bytes(4),

            BaseType::I64 |
            BaseType::U64 |
            BaseType::F64 => MemorySize::from_bytes(8),
        }
    }
    pub fn get_non_struct_memory_size(&self) -> MemorySize {
        match self {
            BaseType::VOID => panic!("tried to get size of void"),
            BaseType::VaArg => panic!("tried to get size of varadic arg"),

            BaseType::STRUCT(_) => panic!("tried to calculate size of struct without an asm_data"),

            BaseType::_BOOL |
            BaseType::I8 |
            BaseType::U8 => MemorySize::from_bytes(1),

            BaseType::I16 |
            BaseType::U16 => MemorySize::from_bytes(2),

            BaseType::I32 |
            BaseType::U32 |
            BaseType::F32 => MemorySize::from_bytes(4),

            BaseType::I64 |
            BaseType::U64 |
            BaseType::F64 => MemorySize::from_bytes(8),
        }
    }
}

impl Display for BaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BaseType::VOID => write!(f, "void"),
            BaseType::VaArg => write!(f, "varadic"),
            BaseType::_BOOL => write!(f, "bool"),
            BaseType::I8 => write!(f, "i8"),
            BaseType::U8 => write!(f, "u8"),
            BaseType::I16 => write!(f, "i16"),
            BaseType::U16 => write!(f, "u16"),
            BaseType::I32 => write!(f, "i32"),
            BaseType::U32 => write!(f, "u32"),
            BaseType::I64 => write!(f, "i64"),
            BaseType::U64 => write!(f, "u64"),
            BaseType::F32 => write!(f, "f32"),
            BaseType::F64 => write!(f, "f64"),
            BaseType::STRUCT(struct_identifier) => write!(f, "{}", struct_identifier),
        }
    }
}

pub fn new_from_type_list(type_info: &[TypeInfo]) -> BaseType {

    //void type
    if type_info.contains(&TypeInfo::VOID){
        return BaseType::VOID;
    }
    //varadic arg
    if type_info.contains(&TypeInfo::VaArg) {
        return BaseType::VaArg;
    }
    //boolean
    if type_info.contains(&TypeInfo::_BOOL) {
        assert!(type_info.len() == 1);
        return BaseType::_BOOL;
    }

    //int assumed from now on
    let unsigned = type_info.contains(&TypeInfo::UNSIGNED);

    let is_long = type_info.contains(&TypeInfo::LONG);
    let is_short = type_info.contains(&TypeInfo::SHORT);
    let is_char = type_info.contains(&TypeInfo::CHAR);

    let size_bytes = match (is_long, is_short, is_char) {
        (true, false, false) => 8,
        (false, false, false) => 4,//default is 32 bit
        (false, true, false) => 2,
        (false, false, true) => 1,
        _ => panic!("unknown type")
    };

    let base_type = match (unsigned, size_bytes) {
        (true, 8) => BaseType::U64,
        (false, 8) => BaseType::I64,
        (true, 4) => BaseType::U32,
        (false, 4) => BaseType::I32,
        (true, 2) => BaseType::U16,
        (false, 2) => BaseType::I16,
        (true, 1) => BaseType::U8,
        (false, 1) => BaseType::I8,

        (_, _) => panic!("unsupported size"),
    };

    base_type
}