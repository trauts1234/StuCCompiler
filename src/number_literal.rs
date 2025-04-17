use std::ops::Neg;
use unwrap_let::unwrap_let;
use crate::{asm_gen_data::AsmData, assembly::operand::immediate::ImmediateValue, data_type::{base_type::BaseType, recursive_data_type::{calculate_unary_type_arithmetic, DataType}}, expression_visitors::expr_visitor::ExprVisitor};

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

    pub fn get_data_type(&self) -> DataType {
        DataType::new(self.data_type.clone())
    }

    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_number_literal(self)
    }
}

impl Neg for NumberLiteral {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let negated_val = match self.value {
            LiteralValue::SIGNED(x) => LiteralValue::SIGNED(-x),
            LiteralValue::UNSIGNED(x) => LiteralValue::UNSIGNED(x.wrapping_neg()),
        };

        unwrap_let!(DataType::RAW(base) = calculate_unary_type_arithmetic(&DataType::RAW(self.data_type.clone())));

        Self {
            value: negated_val,
            data_type: base,
        }
    }
}

impl NumberLiteral {
    //TODO maybe impl From/TryFrom???
    pub fn new(input: &str) -> NumberLiteral {

        let input = input.to_ascii_lowercase();

        let chars: Vec<char> = input.chars().collect();
        let (base, remaining) = match chars.as_slice() {
            ['0', 'x', remaining @ ..] => (16, remaining),
            ['0', 'b', remaining @ ..] => (2, remaining),
            ['0', remaining @ ..] => (8, remaining),
            remaining => (10, remaining)
        };
        let is_integer = !remaining.contains(&'.');

        let (forced_type, remaining): (Option<BaseType>, _) = match remaining {
            //float types

            [x @.., 'f'] if !is_integer => (todo!("float"), x),
            [x @.., 'l'] if !is_integer => (todo!("double"), x),
            x if !is_integer => (todo!("double"), x),

            //integer types

            [x @.., 'u','l','l'] | //ull
            [x @.., 'l', 'l', 'u'] |//llu
            [x @.., 'u','l'] |//ul
            [x @.., 'l', 'u'] //lu
                if is_integer => (Some(BaseType::U64), x),//all are u64

            [x @.., 'u'] if is_integer => (Some(BaseType::U32), x),//u

            [x @.., 'l', 'l' ] | //ll
            [x @.., 'l'] //l
                if is_integer => (Some(BaseType::I64), x),//are i64
            
            x if is_integer => (None, x),//no suffix, predict type based on data size

            _ => panic!("invalid literal")
        };

        let (remaining, literal_exponent) = match base {
            16 => {
                if let Some(pos) = remaining.iter().position(|&c| c == 'p') {
                    let lhs = &remaining[..pos];
                    let power = &remaining[pos+1..];

                    assert!(lhs.len() > 0 && power.len() > 0);

                    (lhs, calculate_positive_integer(base, power))// 0x1.5p3 = 0x1.5 * 2^3
                    
                } else {
                    assert!(is_integer);//hex floats need exponent
                    (remaining, 0)//no power letter
                }
            }

            10 => {
                if let Some(pos) = remaining.iter().position(|&c| c == 'e') {
                    let lhs = &remaining[..pos];
                    let power = &remaining[pos+1..];

                    assert!(lhs.len() > 0 && power.len() > 0);

                    (lhs, calculate_positive_integer(base, power))// 1.5e3 = 1.5 * 10^3
                    
                } else {
                    (remaining, 0)//no power letter
                }
            },

            8 | 2 => (remaining, 0),//no possible exponent for octal or binary numbers

            _ => panic!("invalid base for number literal")
        };

        if is_integer {
            assert!(literal_exponent == 0);//no power on integers
            let as_large_unsigned = calculate_positive_integer(base, remaining);

            let predicted_type = match as_large_unsigned {
                0..=2_147_483_647 => BaseType::I32,
                2_147_483_648..=9_223_372_036_854_775_807 => BaseType::I64,
                9_223_372_036_854_775_808..=18446744073709551615 => BaseType::U64,
            };
    
            let data_type = forced_type.unwrap_or(predicted_type);
            assert!(data_type.is_integer());
    
            NumberLiteral {
                value:LiteralValue::UNSIGNED(as_large_unsigned),//integer literals from text are always positive
                data_type: BaseType::U64,//start as large type, and cast from there
            }.cast(&data_type)//cast to the correct type
        } else {
            todo!("float literals")
        }

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
    pub fn nasm_format(&self) -> ImmediateValue {
        ImmediateValue(match self.value {
            LiteralValue::SIGNED(x) => x.to_string(),
            LiteralValue::UNSIGNED(x) => x.to_string(),
        })
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
        
        number_bytes[..bytes_size as usize].iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(",")
    }

    pub fn cast(&self, new_type: &BaseType) -> NumberLiteral {
        let i64_type: i64 = self.value.clone().into();
        let u64_type: u64 = self.value.clone().into();
        let new_value = match new_type {
            BaseType::I8 => LiteralValue::SIGNED(i64_type as i8 as i64),
            BaseType::I16 => LiteralValue::SIGNED(i64_type as i16 as i64),
            BaseType::I32 => LiteralValue::SIGNED(i64_type as i32 as i64),
            BaseType::I64 => LiteralValue::SIGNED(i64_type),

            BaseType::U8 => LiteralValue::UNSIGNED(u64_type as u8 as u64),
            BaseType::U16 => LiteralValue::UNSIGNED(u64_type as u16 as u64),
            BaseType::U32 => LiteralValue::UNSIGNED(u64_type as u32 as u64),
            BaseType::U64 => LiteralValue::UNSIGNED(u64_type),

            _ => panic!("tried to cast number literal to unknown data type")
        };

        NumberLiteral { value: new_value, data_type: new_type.clone() }
    }
}

impl Into<i64> for LiteralValue {
    fn into(self) -> i64 {
        match self {
            Self::SIGNED(x) => x,
            Self::UNSIGNED(x) => x as i64,
        }
    }
}

impl Into<u64> for LiteralValue {
    fn into(self) -> u64 {
        match self {
            Self::SIGNED(x) => x as u64,
            Self::UNSIGNED(x) => x,
        }
    }
}

fn calculate_positive_integer(base: u64, digits: &[char]) -> u64 {
    assert!(!digits.contains(&'.'));

    digits.iter()
    .map(|c| c.to_digit(base.try_into().unwrap()).unwrap() as u64)//get each digit as a number
    .rev()//go from lowest place value digit first
    .enumerate()
    .map(|(i, digit)| digit * base.pow(i.try_into().unwrap()))//take into account place value of each digit
    .sum()//sum each place value
}
fn calculate_float(base: i32, digits: &[char]) -> f64 {
    assert!(digits.contains(&'.'));
    todo!();
}