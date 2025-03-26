use crate::{asm_gen_data::AsmData, data_type::type_token::TypeInfo, memory_size::MemoryLayout};

use super::recursive_data_type::RecursiveDataType;


#[derive(Debug, Clone, PartialEq)]
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
    STRUCT(String)
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
            BaseType::VOID | BaseType::VaArg | BaseType::STRUCT(_) => false,

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
            BaseType::I64 => false,

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

    pub fn memory_size(&self, asm_data: &AsmData) -> MemoryLayout {
        match self {
            BaseType::VOID => panic!("tried to get size of void"),
            BaseType::VaArg => panic!("tried to get size of varadic arg"),

            BaseType::STRUCT(x) => asm_data.get_struct(x).calculate_size().expect("tried to calculate size of partially declared struct"),

            BaseType::_BOOL |
            BaseType::I8 |
            BaseType::U8 => MemoryLayout::from_bits(8),

            BaseType::I16 |
            BaseType::U16 => MemoryLayout::from_bits(16),

            BaseType::I32 |
            BaseType::U32 => MemoryLayout::from_bits(32),

            BaseType::I64 |
            BaseType::U64 => MemoryLayout::from_bits(64),
        }
    }
}

pub fn new_from_type_list(type_info: &[TypeInfo]) -> BaseType {

    assert!(type_info.len() > 0);

    if type_info.contains(&TypeInfo::EXTERN) {
        println!("extern modifiers not counted. if this function doesn't have a definition it will be automatically marked extern");
    }

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

    let size_bits = match (is_long, is_short, is_char) {
        (true, false, false) => 64,
        (false, false, false) => 32,//default is 32 bit
        (false, true, false) => 16,
        (false, false, true) => 8,
        _ => panic!("unknown type")
    };

    let base_type = match (unsigned, size_bits) {
        (true, 64) => BaseType::U64,
        (false, 64) => BaseType::I64,
        (true, 32) => BaseType::U32,
        (false, 32) => BaseType::I32,
        (true, 16) => BaseType::U16,
        (false, 16) => BaseType::I16,
        (true, 8) => BaseType::U8,
        (false, 8) => BaseType::I8,

        (_, _) => panic!("unsupported size"),
    };

    base_type
}