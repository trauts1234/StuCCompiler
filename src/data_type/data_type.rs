use crate::memory_size::MemoryLayout;

use super::{base_type::BaseType, type_modifier::DeclModifier, type_token::TypeInfo};

/**
 * an entire type to describe anything's type
 */
#[derive(Debug, Clone, PartialEq)]
pub struct DataType {
    base_type: BaseType,
    modifiers: Vec<DeclModifier>,//apply top to bottom of stack, so [POINTER, ARRAY(4)] is an (array of 4) pointer
}

impl DataType {

    pub fn new_from_type_list(type_info: &[TypeInfo], modifiers: &[DeclModifier]) -> DataType {

        if type_info.contains(&TypeInfo::EXTERN) {
            println!("extern modifiers not counted. if this function doesn't have a definition it will be automatically marked extern");
        }

        //void type
        if type_info.contains(&TypeInfo::VOID){
            return DataType { base_type:BaseType::VOID, modifiers:modifiers.to_vec()};
        }
        //varadic arg
        if type_info.contains(&TypeInfo::VaArg) {
            assert!(modifiers.len() == 0);//can't have a va arg pointer???
            return DataType {base_type: BaseType::VaArg, modifiers: Vec::new()};
        }
        //boolean
        if type_info.contains(&TypeInfo::_BOOL) {
            assert!(type_info.len() == 1);
            return DataType {base_type: BaseType::_BOOL, modifiers:modifiers.to_vec()};
        }

        //int assumed from now on
        let unsigned = type_info.contains(&TypeInfo::UNSIGNED);

        //char
        if type_info.contains(&TypeInfo::CHAR) {
            assert!(type_info.len() <= 2);
            let base_type = if unsigned {BaseType::U8} else {BaseType::I8};

            return DataType {base_type, modifiers:modifiers.to_vec()};
        }

        let is_long = type_info.contains(&TypeInfo::LONG);

        if is_long {
            let base_type = if unsigned {BaseType::U64} else {BaseType::I64};

            return DataType {base_type, modifiers:modifiers.to_vec()};
        }

        assert!(type_info.contains(&TypeInfo::INT));

        let base_type = if unsigned {BaseType::U32} else {BaseType::I32};

        return DataType {base_type, modifiers:modifiers.to_vec()};

    }
    pub fn new_from_base_type(base_type: &BaseType, modifiers: &[DeclModifier]) -> DataType {
        DataType { base_type: base_type.clone(), modifiers: modifiers.to_vec() }
    }

    pub fn memory_size(&self) -> MemoryLayout {

        let base_size = self.underlying_type().memory_size();

        //take into account if this is a pointer, array, etc.
        self.modifiers.iter()
        .fold(base_size, |acc,x| match x {
            DeclModifier::POINTER => MemoryLayout::from_bytes(8),//pointer to anything is always 8 bytes
            DeclModifier::ARRAY(arr_elements) => MemoryLayout::from_bytes(acc.size_bytes() * arr_elements),
            DeclModifier::FUNCTION => panic!("tried to calculate size of function???")
        })
    }

    pub fn get_modifiers(&self) -> &[DeclModifier] {
        &self.modifiers
    }

    pub fn underlying_type(&self) -> &BaseType {
        &self.base_type
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
     * tries to decay myself as an array to pointer, or return myself if I can't be decayed
     */
    pub fn decay(&self) -> DataType {
        if self.modifiers.len() == 0{
            return self.clone();
        }

        if let DeclModifier::ARRAY(_) = self.modifiers[0] {
            let mut result = self.clone();
            result.modifiers[0] = DeclModifier::POINTER;//turn the array of x into a pointer to x
            result
        } else {
            self.clone()
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
        if lhs.is_array() {
            return lhs.decay();
        }

        if rhs.is_array() {
            return rhs.decay();
        }

        //see if this is pointer arithmetic
        if lhs.is_pointer() {
            return lhs.clone();
        }
        if rhs.is_pointer() {
            return rhs.clone();
        }

        //todo float managment

        if lhs.underlying_type().is_integer() && rhs.underlying_type().is_integer() {
            //integer type promotion
            let biggest_size = lhs.memory_size().size_bits().max(rhs.memory_size().size_bits());

            return match (biggest_size, lhs.underlying_type().is_unsigned(), rhs.underlying_type().is_unsigned()) {
                (0..=31, _, _) |// small enough to be cast to int easily
                (32, false, false)//signed, and both int sized
                 => DataType::new_from_base_type(&BaseType::I32, &Vec::new()),

                (32, _, _) => DataType::new_from_base_type(&BaseType::U32, &Vec::new()),

                (33..=63, _, _) |// small enough to be cast to long long easily
                (64, false, false)//signed, and both are long long sized
                 => DataType::new_from_base_type(&BaseType::I64, &Vec::new()),

                (64, _, _) //64 bit, with one being unsigned
                => DataType::new_from_base_type(&BaseType::U64, &Vec::new()),

                (65.., _, _) => panic!("integer size too large!")

            };


        }

        panic!();

    }

    pub fn calculate_unary_type_arithmetic(lhs: &DataType) -> DataType {
        //see if it wants to be a pointer
        if lhs.is_array() {
            return lhs.decay();
        }

        //see if this is pointer arithmetic
        if lhs.is_pointer() {
            return lhs.clone();
        }

        //todo float managment

        if lhs.underlying_type().is_integer() {
            //integer type promotion

            return match (lhs.memory_size().size_bits(), lhs.underlying_type().is_unsigned()) {
                (0..=31, _) |// small enough to be cast to int easily
                (32, false)//signed, and both int sized
                 => DataType::new_from_base_type(&BaseType::I32, &Vec::new()),

                (32, true) => DataType::new_from_base_type(&BaseType::U32, &Vec::new()),

                (33..=63, _) |// small enough to be cast to long long easily
                (64, false)//signed, and long long sized
                 => DataType::new_from_base_type(&BaseType::I64, &Vec::new()),

                (64, true) =>  DataType::new_from_base_type(&BaseType::U64, &Vec::new()),

                (65.., _) => panic!("integer size too large!")

            };


        }

        panic!();//integers already handled
    }
}