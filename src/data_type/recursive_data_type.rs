use crate::{asm_gen_data::AsmData, memory_size::MemoryLayout};

use super::{base_type::BaseType, type_modifier::DeclModifier};


#[derive(Clone, Debug, PartialEq)]
pub enum RecursiveDataType {
    ARRAY{size: usize, element: Box<RecursiveDataType>},
    POINTER(Box<RecursiveDataType>),
    RAW(BaseType)
}

impl RecursiveDataType
{
    pub fn new(base: BaseType) -> Self {
        RecursiveDataType::RAW(base)
    }
    pub fn new_from_slice(base: BaseType, items: &[DeclModifier]) -> Self {
        match items {
            //no modifiers left, just raw type
            [] => RecursiveDataType::RAW(base),
            //array of count, and "remaining" tokens => array of count, process(remaining)
            [DeclModifier::ARRAY(count), remaining @ ..] => RecursiveDataType::ARRAY { size: *count, element: Box::new(Self::new_from_slice(base, remaining)) },
            //pointer to "remaining" tokens => pointer to process(remaining)
            [DeclModifier::POINTER, remaining @ ..] => RecursiveDataType::POINTER(Box::new(Self::new_from_slice(base, remaining)))
        }
    }
    
    pub fn decay(&self) -> Self {
        match self {
            Self::ARRAY { size:_, element } => RecursiveDataType::POINTER(element.clone()),
            _ => self.clone()
        }
    }

    pub fn remove_outer_modifier(&self) -> Self {
        match self {
            Self::ARRAY { size:_, element } => *element.clone(),
            Self::POINTER(element) => *element.clone(),
            Self::RAW(_) => panic!("tried to remove outer modifier from raw type")
        }
    }
    pub fn add_outer_modifier(&self, modifier: DeclModifier) -> Self {
        match modifier {
            DeclModifier::POINTER => Self::POINTER(Box::new(self.clone())),
            DeclModifier::ARRAY(size) => Self::ARRAY { size, element: Box::new(self.clone()) },
        }
    }
    pub fn add_inner_modifier(&self, modifier: DeclModifier) -> Self {
        match self {
            RecursiveDataType::ARRAY { size, element } => RecursiveDataType::ARRAY { size: *size, element: Box::new(element.add_inner_modifier(modifier)) },
            RecursiveDataType::POINTER(recursive_data_type) => RecursiveDataType::POINTER(Box::new(recursive_data_type.add_inner_modifier(modifier))),
            RecursiveDataType::RAW(base_type) => Self::new_from_slice(base_type.clone(), &[modifier]),//add the modifier to the innermost
        }
    }

    pub fn memory_size(&self, asm_data: &AsmData) -> MemoryLayout {
        match self {
            RecursiveDataType::ARRAY { size, element } => MemoryLayout::from_bits(size * &element.memory_size(asm_data).size_bits()),
            RecursiveDataType::POINTER(_) => MemoryLayout::from_bytes(8),
            RecursiveDataType::RAW(base) => base.memory_size(asm_data),
        }
    }
}

/**
 * calculates the data type when simple arithmetic is applied to lhs and rhs
 * e.g (float) + (int) => float
 * can be used for the operators add, subtract, multiply, divide
 * also works for pointers
 */
pub fn calculate_promoted_type_arithmetic(lhs: &RecursiveDataType, rhs: &RecursiveDataType, asm_data: &AsmData) -> RecursiveDataType {

    match (lhs, rhs) {
        (RecursiveDataType::ARRAY { size:_, element:_ }, _) => lhs.decay(),//lhs is array, so promoted type is pointer
        (RecursiveDataType::POINTER(_), _) => lhs.clone(),//lhs is pointer, so every possible rhs is cast to pointer

        (_, RecursiveDataType::ARRAY { size:_, element:_ }) => rhs.decay(),
        (_, RecursiveDataType::POINTER(_)) => rhs.clone(),

        (RecursiveDataType::RAW(lhs_base), RecursiveDataType::RAW(rhs_base)) => {
            RecursiveDataType::RAW(calculate_integer_promoted_type(lhs_base, rhs_base))
        }
    }

}

pub fn calculate_integer_promoted_type(lhs: &BaseType, rhs: &BaseType) -> BaseType {
    assert!(lhs.is_integer() && rhs.is_integer());

    //integer type promotion
    let biggest_size = lhs.get_non_struct_memory_size().size_bits().max(rhs.get_non_struct_memory_size().size_bits());

    match (biggest_size, lhs.is_unsigned(), rhs.is_unsigned()) {
        (0..=31, _, _) |// small enough to be cast to int easily
        (32, false, false)//signed, and both int sized
            => BaseType::I32,

        (32, _, _) => BaseType::U32,

        (33..=63, _, _) |// small enough to be cast to long long easily
        (64, false, false)//signed, and both are long long sized
            => BaseType::I64,

        (64, _, _) //64 bit, with one being unsigned
        => BaseType::U64,

        (65.., _, _) => panic!("integer size too large!")

    }
}

pub fn calculate_unary_type_arithmetic(lhs: &RecursiveDataType, asm_data: &AsmData) -> RecursiveDataType {
    match lhs {
        RecursiveDataType::ARRAY { size:_, element:_ } => lhs.decay(),
        RecursiveDataType::POINTER(_) => lhs.clone(),
        RecursiveDataType::RAW(lhs_base) => {
            assert!(lhs_base.is_integer());

            match (lhs_base.memory_size(asm_data).size_bits(), lhs_base.is_unsigned()) {
                (0..=31, _) |// small enough to be cast to int easily
                (32, false)//signed, and both int sized
                    => RecursiveDataType::new(BaseType::I32),
    
                (32, true) => RecursiveDataType::new(BaseType::U32),
    
                (33..=63, _) |// small enough to be cast to long long easily
                (64, false)//signed, and long long sized
                    => RecursiveDataType::new(BaseType::I64),
    
                (64, true) =>  RecursiveDataType::new(BaseType::U64),
    
                (65.., _) => panic!("integer size too large!")
    
            }
        }
    }
}