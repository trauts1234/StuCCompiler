use crate::{asm_gen_data::AsmData, memory_size::MemoryLayout};

use super::{base_type::BaseType, modifier_list::ModifierList, type_token::TypeInfo};

#[derive(Debug, Clone, PartialEq)]
pub struct Primative {
    base_type: BaseType,
    modifiers: ModifierList,
}

impl Primative {
    pub fn new(base_type: BaseType) -> Primative {
        Primative { base_type, modifiers: ModifierList::new() }
    }
    pub fn memory_size(&self) -> MemoryLayout {
        let base_size = self.base_type.memory_size();
        self.modifiers.memory_size(base_size)
    }

    pub fn decay(&self) -> Primative {
        Primative { base_type: self.base_type.clone(), modifiers: self.modifiers.decay() }
    }

    pub fn underlying_type(&self) -> &BaseType {
        &self.base_type
    }
    pub fn get_modifiers(&self) -> &ModifierList {
        &self.modifiers
    }

    pub fn remove_outer_modifier(&self) -> Primative {
        Primative { base_type: self.base_type.clone(), modifiers: self.modifiers.remove_outer_modifier() }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Composite {
    pub(crate) struct_name: String,
    pub(crate) modifiers: ModifierList,
}
impl Composite {
    pub fn new(struct_name: String) -> Composite {
        Composite { struct_name, modifiers: ModifierList::new() }
    }

    pub fn memory_size(&self, asm_data: &AsmData) -> MemoryLayout {
        let base_size = asm_data.get_struct(&self.struct_name).calculate_size().unwrap();
        self.modifiers.memory_size(base_size)
    }

    pub fn decay(&self) -> Composite {
        Composite { struct_name: self.struct_name.clone(), modifiers: self.modifiers.decay() }
    }

    pub fn remove_outer_modifier(&self) -> Composite {
        Composite {struct_name: self.struct_name.clone(), modifiers: self.modifiers.remove_outer_modifier() }
    }
    pub fn get_modifiers(&self) -> &ModifierList {
        &self.modifiers
    }
}

/**
 * an entire type to describe anything's type
 */
#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    PRIMATIVE (Primative),

    COMPOSITE (Composite)
}

impl DataType {

    pub fn make_void() -> DataType {
        Self::new_from_base_type(&BaseType::VOID)
    }

    pub fn new_from_type_list(type_info: &[TypeInfo], modifiers: ModifierList) -> DataType {

        assert!(type_info.len() > 0);

        if type_info.contains(&TypeInfo::EXTERN) {
            println!("extern modifiers not counted. if this function doesn't have a definition it will be automatically marked extern");
        }

        //void type
        if type_info.contains(&TypeInfo::VOID){
            return DataType::PRIMATIVE(Primative{ base_type: BaseType::VOID, modifiers: modifiers })
        }
        //varadic arg
        if type_info.contains(&TypeInfo::VaArg) {
            assert!(modifiers.modifiers_count() == 0);//can't have a va arg pointer???
            return DataType::PRIMATIVE(Primative::new(BaseType::VaArg))
        }
        //boolean
        if type_info.contains(&TypeInfo::_BOOL) {
            assert!(type_info.len() == 1);
            return DataType::PRIMATIVE(Primative{ base_type: BaseType::_BOOL, modifiers });
        }

        //int assumed from now on
        let unsigned = type_info.contains(&TypeInfo::UNSIGNED);

        let is_long = type_info.contains(&TypeInfo::LONG);
        let is_short = type_info.contains(&TypeInfo::SHORT);
        let is_char = type_info.contains(&TypeInfo::CHAR);

        let size_bits = match (is_long, is_short, is_char) {
            (true, false, false) => 64,
            (false, false, false) => 32,//default is 32 bit
            (false, true, false) => 16,
            (false, false, true) => 8,
            _ => panic!("unknown type")
        };

        let base_type = match (unsigned, size_bits) {
            (true, 64) => BaseType::U64,
            (false, 64) => BaseType::I64,
            (true, 32) => BaseType::U32,
            (false, 32) => BaseType::I32,
            (true, 16) => BaseType::U16,
            (false, 16) => BaseType::I16,
            (true, 8) => BaseType::U8,
            (false, 8) => BaseType::I8,

            (_, _) => panic!("unsupported size"),
        };

        DataType::PRIMATIVE( Primative{ base_type, modifiers: modifiers })

    }
    pub fn new_from_base_type(base_type: &BaseType) -> DataType {
        DataType::PRIMATIVE(Primative{ base_type: base_type.clone(), modifiers: ModifierList::new() })
    }

    pub fn memory_size(&self, asm_data: &AsmData) -> MemoryLayout {
        match self {
            DataType::PRIMATIVE(primative) => primative.memory_size(),
            DataType::COMPOSITE(composite) => composite.memory_size(asm_data),
        }
    }

    pub fn get_modifiers(&self) -> &ModifierList {
        match self {
            DataType::PRIMATIVE(primative) => &primative.modifiers,
            DataType::COMPOSITE(composite) => &composite.modifiers,
        }
    }
    pub fn replace_modifiers(&self, new_modifiers: ModifierList) -> DataType {
        match self {
            DataType::PRIMATIVE(Primative { base_type, modifiers: _ }) => DataType::PRIMATIVE(Primative { base_type: base_type.clone(), modifiers: new_modifiers }),
            DataType::COMPOSITE(Composite { struct_name, modifiers: _ }) => DataType::COMPOSITE(Composite { struct_name: struct_name.clone(), modifiers: new_modifiers }),
        }
    }

    pub fn is_pointer(&self) -> bool {
        self.get_modifiers().is_pointer()
    }
    pub fn is_array(&self) -> bool {
        self.get_modifiers().is_array()
    }

    /**
     * tries to decay myself as an array to pointer, or return myself if I can't be decayed
     */
    pub fn decay(&self) -> DataType {
        match self {
            DataType::PRIMATIVE(primative) => DataType::PRIMATIVE(primative.decay()),
            DataType::COMPOSITE(composite) => DataType::COMPOSITE(composite.decay()),
        }
    }

    /**
     * removes the outer layer of modifier
     * e.g int* -> int
     * *int[] -> int* (I think)
     */
    pub fn remove_outer_modifier(&self) -> DataType {
        match self {
            DataType::PRIMATIVE(primative) => DataType::PRIMATIVE(primative.remove_outer_modifier()),
            DataType::COMPOSITE(composite) => DataType::COMPOSITE(composite.remove_outer_modifier()),
        }
    }
    
    /**
     * calculates the data type when simple arithmetic is applied to lhs and rhs
     * e.g (float) + (int) => float
     * can be used for the operators add, subtract, multiply, divide
     * also works for pointers
     */
    pub fn calculate_promoted_type_arithmetic(lhs: &Primative, rhs: &Primative) -> Primative {

        //see if either side wants to be a pointer
        if lhs.get_modifiers().is_array() {
            return lhs.decay();
        }

        if rhs.get_modifiers().is_array() {
            return rhs.decay();
        }

        //see if this is pointer arithmetic
        if lhs.get_modifiers().is_pointer() {
            return lhs.clone();
        }
        if rhs.get_modifiers().is_pointer() {
            return rhs.clone();
        }

        //todo float managment

        if lhs.base_type.is_integer() && rhs.base_type.is_integer() {
            //integer type promotion
            let biggest_size = lhs.memory_size().size_bits().max(rhs.memory_size().size_bits());

            return match (biggest_size, lhs.base_type.is_unsigned(), rhs.base_type.is_unsigned()) {
                (0..=31, _, _) |// small enough to be cast to int easily
                (32, false, false)//signed, and both int sized
                 => Primative::new(BaseType::I32),

                (32, _, _) => Primative::new(BaseType::U32),

                (33..=63, _, _) |// small enough to be cast to long long easily
                (64, false, false)//signed, and both are long long sized
                 => Primative::new(BaseType::I64),

                (64, _, _) //64 bit, with one being unsigned
                => Primative::new(BaseType::U64),

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

        if let DataType::PRIMATIVE(primative) = lhs {
            assert!(primative.base_type.is_integer());
            //integer type promotion

            return match (primative.memory_size().size_bits(), primative.base_type.is_unsigned()) {
                (0..=31, _) |// small enough to be cast to int easily
                (32, false)//signed, and both int sized
                 => DataType::new_from_base_type(&BaseType::I32),

                (32, true) => DataType::new_from_base_type(&BaseType::U32),

                (33..=63, _) |// small enough to be cast to long long easily
                (64, false)//signed, and long long sized
                 => DataType::new_from_base_type(&BaseType::I64),

                (64, true) =>  DataType::new_from_base_type(&BaseType::U64),

                (65.., _) => panic!("integer size too large!")

            };


        }

        panic!();//integers already handled
    }
}