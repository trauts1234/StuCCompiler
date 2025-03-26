use crate::{asm_gen_data::AsmData, memory_size::MemoryLayout};

use super::{base_type::BaseType, type_modifier::DeclModifier};


#[derive(Clone, Debug)]
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

    pub fn memory_size(&self, asm_data: &AsmData) -> MemoryLayout {
        match self {
            RecursiveDataType::ARRAY { size, element } => MemoryLayout::from_bits(size * &element.memory_size(asm_data).size_bits()),
            RecursiveDataType::POINTER(_) => MemoryLayout::from_bytes(8),
            RecursiveDataType::RAW(base) => base.memory_size(asm_data),
        }
    }
}