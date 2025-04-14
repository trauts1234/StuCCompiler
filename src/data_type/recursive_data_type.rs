use crate::{asm_gen_data::AsmData, memory_size::MemoryLayout};

use super::{base_type::BaseType, type_modifier::DeclModifier};


#[derive(Clone, Debug, PartialEq)]
pub enum DataType {
    ARRAY{size: usize, element: Box<DataType>},
    POINTER(Box<DataType>),
    RAW(BaseType)
}

impl DataType
{
    pub fn new(base: BaseType) -> Self {
        DataType::RAW(base)
    }
    pub fn new_from_slice(base: DataType, items: &[DeclModifier]) -> Self {
        match items {
            //no modifiers left, just raw type
            [] => base,
            //array of count, and "remaining" tokens => array of count, process(remaining)
            [DeclModifier::ARRAY(count), remaining @ ..] => DataType::ARRAY { size: *count, element: Box::new(Self::new_from_slice(base, remaining)) },
            //pointer to "remaining" tokens => pointer to process(remaining)
            [DeclModifier::POINTER, remaining @ ..] => DataType::POINTER(Box::new(Self::new_from_slice(base, remaining)))
        }
    }
    
    pub fn decay(&self) -> Self {
        match self {
            Self::ARRAY { size:_, element } => DataType::POINTER(element.clone()),
            _ => self.clone()
        }
    }

    /**
     * if I am a varadic arg, replace myself with to_replace
     */
    pub fn replace_va_arg(&self, to_replace: DataType) -> DataType {
        if DataType::RAW(BaseType::VaArg) == *self {
            to_replace
        } else {
            self.clone()
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
            DataType::ARRAY { size, element } => DataType::ARRAY { size: *size, element: Box::new(element.add_inner_modifier(modifier)) },
            DataType::POINTER(recursive_data_type) => DataType::POINTER(Box::new(recursive_data_type.add_inner_modifier(modifier))),
            DataType::RAW(base_type) => Self::new_from_slice(DataType::RAW(base_type.clone()), &[modifier]),//add the modifier to the innermost
        }
    }

    pub fn memory_size(&self, asm_data: &AsmData) -> MemoryLayout {
        match self {
            DataType::ARRAY { size, element } => MemoryLayout::from_bits(size * &element.memory_size(asm_data).size_bits()),
            DataType::POINTER(_) => MemoryLayout::from_bytes(8),
            DataType::RAW(base) => base.memory_size(asm_data),
        }
    }
}

/**
 * calculates the data type when simple arithmetic is applied to lhs and rhs
 * e.g (float) + (int) => float
 * can be used for the operators add, subtract, multiply, divide
 * also works for pointers
 */
pub fn calculate_promoted_type_arithmetic(lhs: &DataType, rhs: &DataType) -> DataType {

    match (lhs, rhs) {
        (DataType::ARRAY { size:_, element:_ }, _) => lhs.decay(),//lhs is array, so promoted type is pointer
        (DataType::POINTER(_), _) => lhs.clone(),//lhs is pointer, so every possible rhs is cast to pointer

        (_, DataType::ARRAY { size:_, element:_ }) => rhs.decay(),
        (_, DataType::POINTER(_)) => rhs.clone(),

        (DataType::RAW(lhs_base), DataType::RAW(rhs_base)) => {
            DataType::RAW(calculate_integer_promoted_type(lhs_base, rhs_base))
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

/**
 * calculate the type of this type when promoted
 */
pub fn calculate_unary_type_arithmetic(lhs: &DataType, asm_data: &AsmData) -> DataType {
    match lhs {
        DataType::ARRAY { size:_, element:_ } => lhs.decay(),
        DataType::POINTER(_) => lhs.clone(),
        DataType::RAW(lhs_base) => {
            assert!(lhs_base.is_integer());

            match (lhs_base.memory_size(asm_data).size_bits(), lhs_base.is_unsigned()) {
                (0..=31, _) |// small enough to be cast to int easily
                (32, false)//signed, and both int sized
                    => DataType::new(BaseType::I32),
    
                (32, true) => DataType::new(BaseType::U32),
    
                (33..=63, _) |// small enough to be cast to long long easily
                (64, false)//signed, and long long sized
                    => DataType::new(BaseType::I64),
    
                (64, true) =>  DataType::new(BaseType::U64),
    
                (65.., _) => panic!("integer size too large!")
    
            }
        }
    }
}