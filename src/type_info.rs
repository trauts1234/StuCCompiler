use crate::memory_size::MemoryLayout;


#[derive(Debug, Clone, PartialEq)]
pub enum TypeInfo{
    INT,
    UNSIGNED,
    LONG,
    //missing some, should have "static", and other bits that suggest the type of a variable
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeclModifier {
    POINTER,//this declaration is for a pointer to something
    ARRAY(usize),//an array with usize elements
    FUNCTION,//rarely used, preparing for function pointers
}

/**
 * an entire type to describe anything's type
 */
#[derive(Debug, Clone, PartialEq)]
pub struct DataType {
    pub(crate) type_info: Vec<TypeInfo>,
    pub(crate) modifiers: Vec<DeclModifier>//apply top to bottom of stack, so [POINTER, ARRAY(4)] is an (array of 4) pointer
}

impl DataType {
    pub fn memory_size(&self) -> MemoryLayout {

        let base_size = 
        if self.underlying_type_is_integer() {
            if self.underlying_type_is_long_long() {
                MemoryLayout::from_bits(64)//unsigned long long and long long int are both 64 bits
            } else {
                MemoryLayout::from_bits(32)//only i32 is implemented
            }
        } else {
            panic!("not implemented: size of non-integers")
        };

        //take into account if this is a pointer, array, etc.
        self.modifiers.iter()
        .fold(base_size, |acc,x| match x {
            DeclModifier::POINTER => MemoryLayout::from_bytes(8),//pointer to anything is always 8 bytes
            DeclModifier::ARRAY(arr_elements) => MemoryLayout::from_bytes(acc.size_bytes() * arr_elements),
            DeclModifier::FUNCTION => panic!("tried to calculate size of function???")
        })
    }

    /**
     * returns true if the value is any size or type of integer
     * including unsigned
     */
    pub fn underlying_type_is_integer(&self) -> bool {
        true
    }

    pub fn underlying_type_is_unsigned(&self) -> bool {
        self.type_info.contains(&TypeInfo::UNSIGNED)
    }

    pub fn underlying_type_is_long_long(&self) -> bool {
        self.type_info.iter()
            .filter(|typedata| **typedata == TypeInfo::LONG)
            .count() == 2//must contain 2 longs
    }

    pub fn is_pointer(&self) -> bool {
        if self.modifiers.len() == 0{
            return false;
        }
        
        return self.modifiers[0] == DeclModifier::POINTER;//pointer to anything
    }
    pub fn is_array(&self) -> bool {
        if self.modifiers.len() == 0{
            return false;
        }

        if let DeclModifier::ARRAY(_) = self.modifiers[0] {
            return true;
        }

        false
    }

    /**
     * tries to decay myself as an array to pointer, returning None if I am not an array
     */
    pub fn decay_array_to_pointer(&self) -> Option<DataType> {
        if self.modifiers.len() == 0{
            return None;
        }

        if let DeclModifier::ARRAY(_) = self.modifiers[0] {
            let mut result = self.clone();
            result.modifiers[0] = DeclModifier::POINTER;//turn the array of x into a pointer to x
            Some(result)
        } else {
            None
        }
    }

    /**
     * removes the outer layer of modifier
     * e.g int* -> int
     * *int[] -> int* (I think)
     */
    pub fn remove_outer_modifier(&self) -> DataType {
        let mut result = self.clone();

        if result.modifiers.len() > 0 {
            result.modifiers.remove(0);//remove the first modifier
        }

        result
    }
    
    /**
     * calculates the data type when simple arithmetic is applied to lhs and rhs
     * e.g (float) + (int) => float
     * can be used for the operators add, subtract, multiply, divide
     * also works for pointers
     */
    pub fn calculate_promoted_type_arithmetic(lhs: &DataType, rhs: &DataType) -> DataType {

        //see if either side wants to be a pointer
        if let Some(lhs_as_pointer) = lhs.decay_array_to_pointer() {
            return lhs_as_pointer;
        }

        if let Some(rhs_as_pointer) = rhs.decay_array_to_pointer() {
            return rhs_as_pointer;
        }

        //see if this is pointer arithmetic
        if lhs.is_pointer() {
            return lhs.clone();
        }
        if rhs.is_pointer() {
            return rhs.clone();
        }

        //todo float managment

        if lhs.underlying_type_is_unsigned() && rhs.underlying_type_is_unsigned() {
            //unsigned + unsigned is calculated as unsigned
            return DataType {
                type_info: vec![TypeInfo::UNSIGNED, TypeInfo::LONG, TypeInfo::LONG, TypeInfo::INT],//unsigned long long int
                modifiers: Vec::new(),//not an array or pointer as that has already been handled
            };
        }

        return DataType {
            type_info: vec![TypeInfo::LONG, TypeInfo::LONG, TypeInfo::INT],//long long int
            modifiers: Vec::new(),//not an array or pointer as that has already been handled
        };

    }
}

impl TypeInfo {
    pub fn try_new(to_token: &str) -> Option<TypeInfo>{
        match to_token {
            "int" => Some(Self::INT),
            "long" => Some(Self::LONG),
            _ => None
        }
    }
}