use crate::{asm_gen_data::AsmData, data_type::type_token::TypeInfo};
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

    pub fn memory_size(&self, asm_data: &AsmData) -> MemorySize {
        match self {
            BaseType::VOID => panic!("tried to get size of void"),
            BaseType::VaArg => panic!("tried to get size of varadic arg"),

            BaseType::STRUCT(x) => asm_data.get_struct(x).calculate_size().expect("tried to calculate size of partially declared struct"),

            BaseType::_BOOL |
            BaseType::I8 |
            BaseType::U8 => MemorySize::from_bytes(1),

            BaseType::I16 |
            BaseType::U16 => MemorySize::from_bytes(2),

            BaseType::I32 |
            BaseType::U32 => MemorySize::from_bytes(4),

            BaseType::I64 |
            BaseType::U64 => MemorySize::from_bytes(8),
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
            BaseType::U32 => MemorySize::from_bytes(4),

            BaseType::I64 |
            BaseType::U64 => MemorySize::from_bytes(8),
        }
    }
}

pub fn new_from_type_list(type_info: &[TypeInfo]) -> BaseType {

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