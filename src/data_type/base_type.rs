use std::fmt::Display;

use crate::{asm_gen_data::GetStructUnion, data_type::type_token::TypeInfo, struct_definition::StructIdentifier, union_definition::UnionIdentifier};
use memory_size::MemorySize;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum FloatType {
    F32,
    F64,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum IntegerType {
    _BOOL,
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScalarType {
    Float(FloatType),
    Integer(IntegerType)
}

impl IntegerType {
    pub fn memory_size(&self) -> MemorySize {
        match self {
            IntegerType::_BOOL |
            IntegerType::I8 |
            IntegerType::U8 => MemorySize::from_bytes(1),
            IntegerType::I16 |
            IntegerType::U16 => MemorySize::from_bytes(2),
            IntegerType::I32 |
            IntegerType::U32 => MemorySize::from_bytes(4),
            IntegerType::I64 |
            IntegerType::U64 => MemorySize::from_bytes(8),
        }
    }
    pub fn is_unsigned(&self) -> bool {
        match self {
            Self::I8 | 
            Self::I16 | 
            Self::I32 | 
            Self::I64 => false,

            Self::_BOOL |
            Self::U8 | 
            Self::U16 | 
            Self::U32 | 
            Self::U64 => true,
        }
    }
}
impl FloatType {
    pub fn memory_size(&self) -> MemorySize {
        match self {
            FloatType::F32 => MemorySize::from_bytes(4),
            FloatType::F64 => MemorySize::from_bytes(8),
        }
    }
}
impl ScalarType {
    pub fn memory_size(&self) -> MemorySize {
        match self {
            ScalarType::Float(float_type) => float_type.memory_size(),
            ScalarType::Integer(integer_type) => integer_type.memory_size(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaseType {
    VOID,
    VaArg,//varadic arg has a special type as it has no type?
    Scalar(ScalarType),
    Struct(StructIdentifier),
    Union(UnionIdentifier),
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
            BaseType::Scalar(ScalarType::Integer(_)) => true,
            _ => false
        }
    }
    
    pub fn is_unsigned(&self) -> bool {
        match self {
            BaseType::Scalar(ScalarType::Integer(x)) => x.is_unsigned(),
            BaseType::Scalar(ScalarType::Float(_)) => false,
            _ => panic!("can't tell whether this thing is unsigned?")
        }
    }
    pub fn is_signed(&self) -> bool {
        !self.is_unsigned()
    }

    pub fn memory_size(&self, struct_info: &dyn GetStructUnion) -> MemorySize {
        match self {
            BaseType::VOID => panic!("tried to get size of void"),
            BaseType::VaArg => panic!("tried to get size of varadic arg"),

            BaseType::Struct(x) => struct_info.get_struct(x).calculate_size().expect("tried to calculate size of partially declared struct"),
            BaseType::Union(x) => struct_info.get_union(x).calculate_size(struct_info).expect("tried to calculate size of partially declared union"),

            BaseType::Scalar(s) => s.memory_size()
        }
    }
    pub fn get_non_struct_memory_size(&self) -> MemorySize {
        match self {
            BaseType::VOID => panic!("tried to get size of void"),
            BaseType::VaArg => panic!("tried to get size of varadic arg"),
            BaseType::Struct(_) => panic!("tried to calculate size of struct without an asm_data"),
            BaseType::Union(_) => panic!("tried to calculate size of union withouat an asm_data"),

            BaseType::Scalar(s) => s.memory_size()
        }
    }
}

impl Display for BaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BaseType::VOID => write!(f, "void"),
            BaseType::VaArg => write!(f, "varadic"),
            BaseType::Scalar(s) => write!(f, "{}", s),
            BaseType::Struct(struct_identifier) => write!(f, "{}", struct_identifier),
            BaseType::Union(union_identifier) => write!(f, "{}", union_identifier),
        }
    }
}
impl Display for IntegerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::_BOOL => write!(f, "bool"),
            Self::I8 => write!(f, "i8"),
            Self::U8 => write!(f, "u8"),
            Self::I16 => write!(f, "i16"),
            Self::U16 => write!(f, "u16"),
            Self::I32 => write!(f, "i32"),
            Self::U32 => write!(f, "u32"),
            Self::I64 => write!(f, "i64"),
            Self::U64 => write!(f, "u64"),
        }
    }
}
impl Display for FloatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::F32 => write!(f, "f32"),
            Self::F64 => write!(f, "f64"),
        }
    }
}
impl Display for ScalarType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScalarType::Float(float_type) => write!(f, "{}", float_type),
            ScalarType::Integer(integer_type) => write!(f, "{}", integer_type),
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
        return BaseType::Scalar(ScalarType::Integer(IntegerType::_BOOL));
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

    if type_info.contains(&TypeInfo::DOUBLE) {
        assert!(!unsigned);//can't have unsigned double
        BaseType::Scalar(ScalarType::Float(FloatType::F64))
    } else if type_info.contains(&TypeInfo::FLOAT) {
        assert!(!unsigned);//can't have unsigned float
        BaseType::Scalar(ScalarType::Float(FloatType::F32))
    } else {
        match (unsigned, size_bytes) {
            (true, 8) => BaseType::Scalar(ScalarType::Integer(IntegerType::U64)),
            (false, 8) => BaseType::Scalar(ScalarType::Integer(IntegerType::I64)),
            (true, 4) => BaseType::Scalar(ScalarType::Integer(IntegerType::U32)),
            (false, 4) => BaseType::Scalar(ScalarType::Integer(IntegerType::I32)),
            (true, 2) => BaseType::Scalar(ScalarType::Integer(IntegerType::U16)),
            (false, 2) => BaseType::Scalar(ScalarType::Integer(IntegerType::I16)),
            (true, 1) => BaseType::Scalar(ScalarType::Integer(IntegerType::U8)),
            (false, 1) => BaseType::Scalar(ScalarType::Integer(IntegerType::I8)),

            (_, _) => panic!("unsupported size"),
        }
    }

}