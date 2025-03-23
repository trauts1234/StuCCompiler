use crate::{memory_size::MemoryLayout, struct_definition::StructDefinition};


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
    STRUCT(StructDefinition)//could be a partial definition
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

    pub fn is_struct(&self) -> bool {
        match self {
            BaseType::STRUCT(_) => true,
            _ => false
        }
    }

    pub fn memory_size(&self) -> MemoryLayout {
        match self {
            BaseType::VOID => panic!("tried to get size of void"),
            BaseType::VaArg => panic!("tried to get size of varadic arg"),

            BaseType::STRUCT(x) => x.calculate_size().expect("tried to calculate size of partially declared struct"),

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