use crate::{asm_gen_data::AsmData, data_type::{base_type::BaseType, recursive_data_type::RecursiveDataType}, expression_visitors::expr_visitor::ExprVisitor};

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    SIGNED(i64),//big enough for any signed type
    UNSIGNED(u64),// '' unsigned type
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberLiteral {
    value: LiteralValue,
    data_type: BaseType
}

impl NumberLiteral {

    pub fn get_data_type(&self) -> RecursiveDataType {
        RecursiveDataType::new(self.data_type.clone())
    }

    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_number_literal(self)
    }
}

impl NumberLiteral {
    pub fn new(to_token: &str) -> NumberLiteral {

        if to_token.starts_with("0") && to_token.len() > 1 {
            panic!("octal numbers are not supported");
        }
        if to_token.starts_with("-0") && to_token.len() > 2 {
            panic!("negative octal numbers are not supported")
        }

        if to_token.contains("-") {
            panic!("negative number literals are not allowed. use unary negation on a positive number if possible")
        }

        if to_token.contains(".") {
            //handle float or double
            let _is_float = to_token.ends_with("f");

            panic!("floating point number literals not supported");
        }

        //integer type

        let without_modifiers = to_token.to_lowercase().trim_end_matches(|c| c == 'u' || c == 'l').to_string();
        let as_large_unsigned: u64 = without_modifiers.parse().unwrap();

        let predicted_type = match as_large_unsigned {
            0..=2_147_483_647 => BaseType::I32,
            2_147_483_648..=9_223_372_036_854_775_807 => BaseType::I64,
            9_223_372_036_854_775_808..=18446744073709551615 => BaseType::U64,
        };

        let is_unsigned = to_token.to_lowercase().contains("u");
        let is_long = to_token.to_lowercase().contains("l");//in linux long is 64 bit as well as ll

        let forced_type: Option<_> = match (is_unsigned, is_long) {
            (true, true) => Some(BaseType::U64),
            (false, true) => Some(BaseType::I64),
            (true, false) => Some(BaseType::U32),
            (false, false) => None//no type annotations, must predict
        };

        let data_type = forced_type.unwrap_or(predicted_type);
        assert!(data_type.is_integer());

        NumberLiteral {
            value:LiteralValue::UNSIGNED(as_large_unsigned),//integer literals from text are always positive
            data_type: BaseType::U64,//start as large type, and cast from there
        }.cast(&data_type)//cast to the correct type
    }

    pub fn new_from_literal_value(value: LiteralValue) -> NumberLiteral {
        let data_type = match &value {
            LiteralValue::SIGNED(_) => BaseType::I64,
            LiteralValue::UNSIGNED(_) => BaseType::U64,
        };

        NumberLiteral{
            value,
            data_type
        }
    }

    pub fn get_value(&self) -> &LiteralValue {
        &self.value
    }

    /**
     * format this number in a way that it can be pasted into a nasm file
     */
    pub fn nasm_format(&self) -> String {
        match self.value {
            LiteralValue::SIGNED(x) => x.to_string(),
            LiteralValue::UNSIGNED(x) => x.to_string(),
        }
    }

    pub fn get_comma_separated_bytes(&self, asm_data: &AsmData) -> String {
        let bytes_size = self.data_type.memory_size(asm_data).size_bytes();//pass in blank

        let number_bytes = match self.value {
            LiteralValue::SIGNED(x) => {
                assert!(self.data_type.is_signed());
                x.to_le_bytes()
            },
            LiteralValue::UNSIGNED(x) => {
                assert!(self.data_type.is_unsigned());
                x.to_le_bytes()
            },
        };
        
        number_bytes[..bytes_size].iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(",")
    }

    pub fn as_usize(&self) -> usize {
        match self.value {
            LiteralValue::SIGNED(x) => {
                assert!(x >= 0);
                x as usize
            }
            LiteralValue::UNSIGNED(x) => x as usize
        }
    }

    pub fn cast(&self, new_type: &BaseType) -> NumberLiteral {
        let new_value = match new_type {
            BaseType::I8 => LiteralValue::SIGNED(self.value.as_i64() as i8 as i64),
            BaseType::I16 => LiteralValue::SIGNED(self.value.as_i64() as i16 as i64),
            BaseType::I32 => LiteralValue::SIGNED(self.value.as_i64() as i32 as i64),
            BaseType::I64 => LiteralValue::SIGNED(self.value.as_i64()),

            BaseType::U8 => LiteralValue::UNSIGNED(self.value.as_u64() as u8 as u64),
            BaseType::U16 => LiteralValue::UNSIGNED(self.value.as_u64() as u16 as u64),
            BaseType::U32 => LiteralValue::UNSIGNED(self.value.as_u64() as u32 as u64),
            BaseType::U64 => LiteralValue::UNSIGNED(self.value.as_u64()),

            _ => panic!("tried to cast number literal to unknown data type")
        };

        NumberLiteral { value: new_value, data_type: new_type.clone() }
    }
}

impl LiteralValue {
    fn as_i64(&self) -> i64 {
        match self {
            Self::SIGNED(x) => *x,
            Self::UNSIGNED(x) => *x as i64,
        }
    }
    fn as_u64(&self) -> u64 {
        match self {
            Self::SIGNED(x) => *x as u64,
            Self::UNSIGNED(x) => *x,
        }
    }
}