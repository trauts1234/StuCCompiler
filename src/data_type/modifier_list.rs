use crate::memory_size::MemoryLayout;

use super::type_modifier::DeclModifier;

#[derive(Debug, Clone, PartialEq)]
pub struct ModifierList {
    modifiers: Vec<DeclModifier>
}
/**
 * getters for information in the modifiers
 */
impl ModifierList {
    pub fn is_array(&self) -> bool {
        if let [DeclModifier::ARRAY(_), ..] = self.modifiers.as_slice() {
            true
        } else {
            false
        }
    }
    pub fn is_pointer(&self) -> bool {
        if let [DeclModifier::POINTER, ..] = self.modifiers.as_slice() {
            true
        } else {
            false
        }
    }
    pub fn modifiers_count(&self) -> usize {
        self.modifiers.len()
    }
    pub fn raw_modifiers(&self) -> &[DeclModifier] {
        &self.modifiers
    }
}

impl ModifierList {
    pub fn new() -> ModifierList {
        ModifierList { modifiers: Vec::new() }
    }
    pub fn new_from_slice(items: &[DeclModifier]) -> ModifierList {
        ModifierList { modifiers: items.to_vec() }
    }
    
    pub fn decay(&self) -> ModifierList {
        let new_modifiers = self.modifiers.iter()
        .enumerate()
        .map(|(i, item)| 
            if i == 0 {
                DeclModifier::POINTER//first item replaced with pointer, so it is a pointer to something
            } else {item.clone()}
        )
        .collect();
        
        ModifierList { modifiers: new_modifiers }
    }

    pub fn remove_outer_modifier(&self) -> ModifierList {
        ModifierList{modifiers: self.modifiers.iter().skip(1).cloned().collect() }
    }
    pub fn add_outer_modifier(&self, modifier: DeclModifier) -> ModifierList {
        ModifierList {
            modifiers: std::iter::once(modifier).chain(self.modifiers.iter().cloned()).collect()
        }
    }

    pub fn memory_size(&self, base_size: MemoryLayout) -> MemoryLayout {
        //take into account if this is a pointer, array, etc.
        self.modifiers.iter().rev()//reverse to start with base type and apply each modifier in turn
        .fold(base_size, |acc,x| match x {
            DeclModifier::POINTER => MemoryLayout::from_bytes(8),//pointer to anything is always 8 bytes
            DeclModifier::ARRAY(arr_elements) => MemoryLayout::from_bytes(acc.size_bytes() * arr_elements),
        })
    }
}