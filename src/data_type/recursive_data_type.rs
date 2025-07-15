use std::fmt::{Debug, Display};

use crate::{asm_gen_data::GetStruct, data_type::base_type::{FloatType, IntegerType, ScalarType}};
use memory_size::MemorySize;
use unwrap_let::unwrap_let;
use super::{base_type::BaseType, type_modifier::DeclModifier};


#[derive(Clone, Debug, PartialEq)]
pub enum DataType {
    UNKNOWNSIZEARRAY{element: Box<DataType>},
    ARRAY{size: u64, element: Box<DataType>},
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
            [DeclModifier::POINTER, remaining @ ..] => DataType::POINTER(Box::new(Self::new_from_slice(base, remaining))),
            [DeclModifier::UnknownSizeArray, remaining @ ..] => DataType::UNKNOWNSIZEARRAY { element: Box::new(Self::new_from_slice(base, remaining)) }
        }
    }
    
    pub fn decay(&self) -> Self {
        match self {
            Self::ARRAY { size:_, element } => DataType::POINTER(element.clone()),
            _ => self.clone()
        }
    }

    /// converts arrays to u64 memory addresses
    /// pointers to u64
    /// any raw type is unaffected
    pub fn decay_to_primative(&self) -> ScalarType {
        match self {
            DataType::UNKNOWNSIZEARRAY { .. } => ScalarType::Integer(IntegerType::U64),
            DataType::ARRAY { .. } => ScalarType::Integer(IntegerType::U64),
            DataType::POINTER(_) => ScalarType::Integer(IntegerType::U64),
            DataType::RAW(BaseType::Scalar(s)) => s.clone(),
            DataType::RAW(bt) => panic!("{:?} base type can't be converted to a primative scalar", bt)
        }
    }

    /// flattens array of arrays into just one giant array
    /// 
    /// # examples
    /// ```
    ///use stuccompiler2::data_type::base_type::BaseType;
    ///use stuccompiler2::data_type::recursive_data_type::DataType;
    /// 
    ///let my_type = DataType::ARRAY {
    ///   size: 2,
    ///   element: Box::new(DataType::ARRAY {
    ///     size: 3,
    ///     element: Box::new(
    ///       DataType::RAW(BaseType::I32)
    ///     )
    ///   })
    /// };
    /// assert_eq!(
    ///   my_type.flatten_nested_array(),
    ///   DataType::ARRAY{
    ///     size:6,
    ///     element: Box::new(DataType::RAW(BaseType::I32))
    ///   }
    /// );
    /// ```
    pub fn flatten_nested_array(&self) -> Self {
        if let DataType::ARRAY { size, element } = self {
            match element.flatten_nested_array() {
                DataType::ARRAY { size: inner_size, element: inner_element } => {
                    DataType::ARRAY {
                        size: size * inner_size,
                        element: inner_element,
                    }
                }
                other => DataType::ARRAY {
                    size: *size,
                    element: Box::new(other),
                },
            }
        } else {
            self.clone()
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
            Self::UNKNOWNSIZEARRAY { element } => *element.clone(),
            Self::ARRAY { size:_, element } => *element.clone(),
            Self::POINTER(element) => *element.clone(),
            Self::RAW(_) => panic!("tried to remove outer modifier from raw type")
        }
    }
    pub fn add_outer_modifier(&self, modifier: DeclModifier) -> Self {
        match modifier {
            DeclModifier::POINTER => Self::POINTER(Box::new(self.clone())),
            DeclModifier::ARRAY(size) => Self::ARRAY { size, element: Box::new(self.clone()) },
            DeclModifier::UnknownSizeArray => Self::UNKNOWNSIZEARRAY { element: Box::new(self.clone()) },
        }
    }

    pub fn memory_size(&self, struct_info: &dyn GetStruct) -> MemorySize {
        match self {
            DataType::UNKNOWNSIZEARRAY { .. } => panic!("cannot find size of unknow size array. perhaps this should return an Option???"),
            DataType::ARRAY { size, element } => MemorySize::from_bytes(size * &element.memory_size(struct_info).size_bytes()),
            DataType::POINTER(_) => MemorySize::from_bytes(8),
            DataType::RAW(base) => base.memory_size(struct_info),
        }
    }

    ///how many RAW or POINTER items are contained in this data type
    ///
    /// # examples
    /// ```
    /// use data_type::recursive_data_type::DataType;
    /// use stuccompiler2::data_type::base_type::BaseType;
    /// 
    /// let my_type = DataType::ARRAY {
    ///   size: 2,
    ///   element: Box::new(DataType::ARRAY {
    ///     size: 3,
    ///     element: Box::new(
    ///       DataType::RAW(BaseType::I32)
    ///     )
    ///   })
    /// };
    /// //since my_type represents int x[2][3];, there are 6 elements
    /// assert_eq!(my_type.array_num_elements(), 6);
    /// 
    /// 
    /// let my_type_2 = DataType::RAW(BaseType::I32);
    /// 
    /// assert_eq!(my_type_2.array_num_elements(), 1);//just an int is the same as an array with 1 element
    /// ```
    pub fn array_num_elements(&self) -> u64 {
        match self {
            DataType::ARRAY { size, element } => size * element.array_num_elements(),
            _ => 1
        }
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}",
        match self {
            DataType::UNKNOWNSIZEARRAY { element } => format!("ARR[]({})", element),
            DataType::ARRAY { size, element } => format!("ARR[{}]({})", size, element),
            DataType::POINTER(data_type) => format!("PTR({})", data_type),
            DataType::RAW(base_type) => format!("{}",base_type),
        })
    }
}

/**
 * calculates the data type when simple arithmetic is applied to lhs and rhs
 * e.g (float) + (int) => float
 * can be used for the operators add, subtract, multiply, divide
 * also works for pointers
 */
pub fn calculate_promoted_type_arithmetic(lhs: &DataType, rhs: &DataType) -> DataType {

    match (lhs.decay(), rhs.decay()) {
        (DataType::POINTER(_), _) => lhs.decay(),//lhs decays to pointer, so every possible rhs is cast to pointer

        (_, DataType::POINTER(_)) => rhs.decay(),

        (DataType::RAW(lhs_base), DataType::RAW(rhs_base)) => {
            unwrap_let!(BaseType::Scalar(lhs_scalar) = lhs_base);
            unwrap_let!(BaseType::Scalar(rhs_scalar) = rhs_base);
            DataType::RAW(BaseType::Scalar(calculate_promoted_type(&lhs_scalar, &rhs_scalar)))
        }

        _ => panic!()//this should never happen as decay always returns pointer or raw
    }

}

pub fn calculate_promoted_type(lhs: &ScalarType, rhs: &ScalarType) -> ScalarType {

    match (lhs, rhs) {
        (ScalarType::Float(FloatType::F64), _) |
        (_, ScalarType::Float(FloatType::F64)) => ScalarType::Float(FloatType::F64),

        (ScalarType::Float(FloatType::F32), _) |
        (_, ScalarType::Float(FloatType::F32)) => ScalarType::Float(FloatType::F32),

        (ScalarType::Integer(lhs_int), ScalarType::Integer(rhs_int)) => {
            //integer type promotion
            let biggest_size = lhs_int.memory_size().max(rhs_int.memory_size());

            ScalarType::Integer(match (biggest_size.size_bytes(), lhs_int.is_unsigned(), rhs_int.is_unsigned()) {
                (0..4, _, _) |// small enough to be cast to int easily
                (4, false, false)//signed, and both int sized
                    => IntegerType::I32,

                (4, _, _) => IntegerType::U32,

                (5..8, _, _) |// small enough to be cast to long long easily
                (8, false, false)//signed, and both are long long sized
                    => IntegerType::I64,

                (8, _, _) //64 bit, with one being unsigned
                => IntegerType::U64,

                (9.., _, _) => panic!("integer size too large!")

            })
        }
    }
}

/**
 * calculate the type of this type when promoted
 */
pub fn calculate_unary_type_arithmetic(lhs: &DataType) -> DataType {
    match lhs.decay() {
        DataType::POINTER(_) => lhs.clone(),
        
        DataType::RAW(BaseType::Scalar(x)) => DataType::RAW(BaseType::Scalar(calculate_unary_type(&x))),

        DataType::RAW(BaseType::Struct(x)) => DataType::RAW(BaseType::Struct(x)),//structs do not need promoting

        x => panic!("what is {:?}", x)//should never happen as decay always returns pointer or raw
    }
}

pub fn calculate_unary_type(lhs: &ScalarType) -> ScalarType {
    match lhs {
        ScalarType::Integer(lhs_base) => ScalarType::Integer(
            match (lhs_base.memory_size().size_bytes(), lhs_base.is_unsigned()) {
                (0..4, _) |// small enough to be cast to int easily
                (4, false)//signed, and both int sized
                    => IntegerType::I32,
    
                (4, true) => IntegerType::U32,
    
                (5..8, _) |// small enough to be cast to long long easily
                (8, false)//signed, and long long sized
                    => IntegerType::I64,
    
                (8, true) =>  IntegerType::U64,
    
                (9.., _) => panic!("integer size too large!")
    
            }
        ),

        ScalarType::Float(lhs_base) => {
            match lhs_base.memory_size().size_bytes() {
                4 => ScalarType::Float(FloatType::F32),
                8 => ScalarType::Float(FloatType::F64),
                _ => panic!("unsupported float size")
            }
        }
    }
}