use crate::memory_size::MemoryLayout;


#[derive(Debug, Clone, PartialEq)]
pub enum TypeInfo{
    INT,
    CHAR,
    _BOOL,
    UNSIGNED,
    LONG,
    EXTERN,
    VOID,

    VaArg,//varadic arg has a special type
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
    pub(crate) modifiers: Vec<DeclModifier>,//apply top to bottom of stack, so [POINTER, ARRAY(4)] is an (array of 4) pointer
}

impl DataType {
    pub fn memory_size(&self) -> MemoryLayout {

        let base_size = self.calculate_base_type_size();

        //take into account if this is a pointer, array, etc.
        self.modifiers.iter()
        .fold(base_size, |acc,x| match x {
            DeclModifier::POINTER => MemoryLayout::from_bytes(8),//pointer to anything is always 8 bytes
            DeclModifier::ARRAY(arr_elements) => MemoryLayout::from_bytes(acc.size_bytes() * arr_elements),
            DeclModifier::FUNCTION => panic!("tried to calculate size of function???")
        })
    }

    pub fn is_void(&self) -> bool {
        if self.type_info.len() == 0 {
            true
        } else if self.type_info.contains(&TypeInfo::VOID) {
            true
        } else {
            false
        }
    }

    pub fn is_varadic_param(&self) -> bool {
        if self.type_info.contains(&TypeInfo::VaArg) {
            assert!(self.modifiers.len() == 0);//can't have a pointer to va arg??
            assert!(self.type_info.len() == 1);//can't be an unsigned va arg or something??

            true
        } else {
            false
        }
    }

    /**
     * returns true if the value is any size or type of integer
     * including unsigned
     */
    pub fn underlying_type_is_integer(&self) -> bool {
        !self.type_info.contains(&TypeInfo::VOID)//can't be void
    }

    pub fn underlying_type_is_unsigned(&self) -> bool {
        self.type_info.contains(&TypeInfo::UNSIGNED)
    }

    pub fn underlying_type_is_long_long(&self) -> bool {
        //in Linux, using LP64, long int is 64 bit
        //but on Windows using ILP64, long int is 32 bit (you need long long int)
        self.type_info.iter()
            .filter(|typedata| **typedata == TypeInfo::LONG)
            .count() >= 1//must contain at least 1 long
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

        if lhs.underlying_type_is_integer() && rhs.underlying_type_is_integer() {
            //integer type promotion
            let biggest_size = lhs.memory_size().size_bits().max(rhs.memory_size().size_bits());

            return match (biggest_size, lhs.underlying_type_is_unsigned(), rhs.underlying_type_is_unsigned()) {
                (0..=31, _, _) |// small enough to be cast to int easily
                (32, false, false)//signed, and both int sized
                 => {
                    DataType {
                        type_info: vec![TypeInfo::INT],
                        modifiers: Vec::new(),//not an array or pointer as that has already been handled
                    }
                },

                (32, _, _) => {
                    //32 bit, with one being unsigned
                    //use unsigned 32 bit
                    DataType {
                        type_info: vec![TypeInfo::UNSIGNED, TypeInfo::INT],
                        modifiers: Vec::new(),//not an array or pointer as that has already been handled
                    }
                },

                (33..=63, _, _) |// small enough to be cast to long long easily
                (64, false, false)//signed, and both are long long sized
                 => {
                    DataType {
                        type_info: vec![TypeInfo::LONG, TypeInfo::LONG, TypeInfo::INT],
                        modifiers: Vec::new(),
                    }
                },

                (64, _, _) => {
                    //64 bit, with one being unsigned
                    //use unsigned 64 bit
                    DataType {
                        type_info: vec![TypeInfo::UNSIGNED, TypeInfo::INT],
                        modifiers: Vec::new(),//not an array or pointer as that has already been handled
                    }
                },

                (65.., _, _) => panic!("integer size too large!")

            };


        }

        panic!();

    }

    pub fn calculate_unary_type_arithmetic(lhs: &DataType) -> DataType {
        //see if it wants to be a pointer
        if let Some(lhs_as_pointer) = lhs.decay_array_to_pointer() {
            return lhs_as_pointer;
        }

        //see if this is pointer arithmetic
        if lhs.is_pointer() {
            return lhs.clone();
        }

        //todo float managment

        if lhs.underlying_type_is_integer() {
            //integer type promotion

            return match (lhs.memory_size().size_bits(), lhs.underlying_type_is_unsigned()) {
                (0..=31, _) |// small enough to be cast to int easily
                (32, false)//signed, and both int sized
                 => {
                    DataType {
                        type_info: vec![TypeInfo::INT],
                        modifiers: Vec::new(),//not an array or pointer as that has already been handled
                    }
                },

                (32, true) => { //u32
                    //use unsigned 32 bit
                    DataType {
                        type_info: vec![TypeInfo::UNSIGNED, TypeInfo::INT],
                        modifiers: Vec::new(),//not an array or pointer as that has already been handled
                    }
                },

                (33..=63, _) |// small enough to be cast to long long easily
                (64, false)//signed, and both are long long sized
                 => {
                    DataType {
                        type_info: vec![TypeInfo::LONG, TypeInfo::LONG, TypeInfo::INT],
                        modifiers: Vec::new(),
                    }
                },

                (64, true) => { //u64
                    //use unsigned 64 bit
                    DataType {
                        type_info: vec![TypeInfo::UNSIGNED, TypeInfo::INT],
                        modifiers: Vec::new(),//not an array or pointer as that has already been handled
                    }
                },

                (65.., _) => panic!("integer size too large!")

            };


        }

        panic!();//integers already handled
    }

    fn calculate_base_type_size(&self) -> MemoryLayout {
        if self.is_void() {
            return MemoryLayout::from_bits(0);
        }
        if self.underlying_type_is_integer() {
            if self.underlying_type_is_long_long() {
                return MemoryLayout::from_bits(64);//unsigned long long and long long int are both 64 bits
            }

            if self.type_info.contains(&TypeInfo::CHAR) {
                return MemoryLayout::from_bits(8);//signed and unsigned char are both 8 bits
            }

            if self.type_info.contains(&TypeInfo::_BOOL) {
                return MemoryLayout::from_bits(8);//bool is 8 bits (only lsb is used)
            }
            
            return MemoryLayout::from_bits(32);//i32 assumed here
        } else {
            panic!("not implemented: size of non-integers")
        }
    }
}

impl TypeInfo {
    pub fn try_new(to_token: &str) -> Option<TypeInfo>{
        match to_token {
            "unsigned" => Some(Self::UNSIGNED),
            "int" => Some(Self::INT),
            "long" => Some(Self::LONG),
            "char" => Some(Self::CHAR),
            "_Bool" => Some(Self::_BOOL),
            "extern" => Some(Self::EXTERN),
            "void" => Some(Self::VOID),
            _ => None
        }
    }
}