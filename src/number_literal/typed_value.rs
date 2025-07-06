use std::{arch::x86_64, cmp::{self, Ordering}, fmt::Display, i128, ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Rem, Shl, Shr, Sub}};
use colored::Colorize;
use unwrap_let::unwrap_let;
use crate::{asm_gen_data::GetStruct, assembly::comparison::ComparisonKind, data_type::{base_type::BaseType, recursive_data_type::{calculate_promoted_type_arithmetic, calculate_unary_type_arithmetic, DataType}}, expression_visitors::expr_visitor::ExprVisitor};

use super::literal_value::LiteralValue;

#[derive(Debug, Clone)]
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

    /// promotes the value using unary promotion rules
    pub fn unary_promote(&self) -> Self {
        unwrap_let!(DataType::RAW(base) = calculate_unary_type_arithmetic(&DataType::RAW(self.data_type.clone())));
        self.cast(&base)
    }

    pub fn binary_promote(self, rhs: Self) -> (Self, Self) {
        unwrap_let!(DataType::RAW(base) = calculate_promoted_type_arithmetic(&DataType::RAW(self.data_type.clone()), &DataType::RAW(rhs.data_type.clone())));

        (
            self.cast(&base),
            rhs.cast(&base)
        )
    }

    pub fn new_from_literal_value(value: LiteralValue) -> NumberLiteral {
        let data_type = match &value {
            LiteralValue::INTEGER(x) if *x < 0 => BaseType::I64,
            _ => BaseType::U64,
        };

        NumberLiteral{
            value,
            data_type
        }
    }

    pub fn get_value(&self) -> &LiteralValue {
        &self.value
    }

    /// Generates the `x db 10` - type commands
    pub fn generate_data_definition_instruction(&self, struct_info: &dyn GetStruct, variable_name: &str) -> String {
        let bytes_size = self.data_type.memory_size(struct_info).size_bytes();//pass in blank

        match self.value {
            LiteralValue::INTEGER(x) => {
                //store the integer as a list of bytes
                format!("{} db {}",
                    variable_name,

                    x.to_le_bytes()[..bytes_size as usize].iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
                )
            },
            
            LiteralValue::FLOAT {value, .. } => {
                match self.data_type {
                    BaseType::F32 => format!("{} dd {:.1}", variable_name, value),
                    BaseType::F64 => format!("{} dq {:.1}", variable_name, value),
                    _ => panic!("invalid data type for float literal?")
                }
            }
        }


    }

    pub fn cast(&self, new_type: &BaseType) -> NumberLiteral {
        let new_value = match (self.value.clone(), new_type) {
            (LiteralValue::INTEGER(val), BaseType::I8)=> LiteralValue::INTEGER(val as i8 as i128),
            (LiteralValue::INTEGER(val), BaseType::I16) => LiteralValue::INTEGER(val as i16 as i128),
            (LiteralValue::INTEGER(val), BaseType::I32) => LiteralValue::INTEGER(val as i32 as i128),
            (LiteralValue::INTEGER(val), BaseType::I64) => LiteralValue::INTEGER(val as i64 as i128),

            (LiteralValue::INTEGER(val), BaseType::U8)=> LiteralValue::INTEGER(val as u8 as i128),
            (LiteralValue::INTEGER(val), BaseType::U16) => LiteralValue::INTEGER(val as u16 as i128),
            (LiteralValue::INTEGER(val), BaseType::U32) => LiteralValue::INTEGER(val as u32 as i128),
            (LiteralValue::INTEGER(val), BaseType::U64) => LiteralValue::INTEGER(val as u64 as i128),

            (LiteralValue::INTEGER(val), BaseType::_BOOL) => LiteralValue::INTEGER(if val == 0 {0} else {1}),//booleans are 1 if nonzero

            _ => panic!("tried to cast number literal to unknown data type")
        };

        NumberLiteral { value: new_value, data_type: new_type.clone() }
    }

    ///ensures that the number stored is a valid number for the data type
    pub fn limit_literal(self) -> Self {
        self.cast(&self.data_type)
    }

    /// promotes the number literal by applying unary plus to it
    pub fn unary_plus(self) -> Self {
        self.unary_promote()
    }

    pub fn boolean_or(self, other: Self) -> Self {
        let promoted = self.unary_promote();

        match Self::binary_promote(self, other) {
            (NumberLiteral { value: LiteralValue::INTEGER(0), data_type: _ }, NumberLiteral { value: LiteralValue::INTEGER(0), data_type: _ }) =>
                Self { value: LiteralValue::INTEGER(0), data_type: promoted.data_type },// 0 | 0 = 0

            _ => Self { value: LiteralValue::INTEGER(1), data_type: promoted.data_type }
        }
    }

    pub fn boolean_and(self, other: Self) -> Self {
        let promoted = self.unary_promote();

        match Self::binary_promote(self, other) {
            (NumberLiteral { value: LiteralValue::INTEGER(x), data_type: _ }, NumberLiteral { value: LiteralValue::INTEGER(y), data_type: _ })
            if x != 0 && y != 0 =>
                Self { value: LiteralValue::INTEGER(1), data_type: promoted.data_type },// 1 | 1 = 1

            _ => {
                Self { value: LiteralValue::INTEGER(0), data_type: promoted.data_type }
            }
        }
    }

    pub fn cmp(self, other: Self, comparison: &ComparisonKind) -> Self {
        let (lhs, rhs) = self.binary_promote(other);

        let cmp_result = match (lhs.value, rhs.value) {
            (LiteralValue::INTEGER(x), LiteralValue::INTEGER(y)) => x.cmp(&y),
            (LiteralValue::INTEGER(x), LiteralValue::FLOAT {value: y, ..}) => cmp_i128_f64(x, y),
            (LiteralValue::FLOAT {value:x,..}, LiteralValue::INTEGER(y)) => cmp_i128_f64(y, x).reverse(),
            (LiteralValue::FLOAT {value:x,..}, LiteralValue::FLOAT {value:y,..}) => x.partial_cmp(&y).unwrap(),
        };

        let result = 
            match comparison {
                ComparisonKind::EQ => LiteralValue::INTEGER(cmp_result.is_eq() as i128),
                ComparisonKind::NE => LiteralValue::INTEGER(cmp_result.is_ne() as i128),
                ComparisonKind::L => LiteralValue::INTEGER(cmp_result.is_lt() as i128),
                ComparisonKind::LE => LiteralValue::INTEGER(cmp_result.is_le() as i128),
                ComparisonKind::G => LiteralValue::INTEGER(cmp_result.is_gt() as i128),
                ComparisonKind::GE => LiteralValue::INTEGER(cmp_result.is_ge() as i128),
                ComparisonKind::ALWAYS => panic!("invalid comparison")
            };

        NumberLiteral {
            value: result,
            data_type: BaseType::_BOOL, // Comparisons result in a boolean type
        }
    }

    pub fn bitwise_not(self) -> Self {
        let mut promoted = self.unary_promote();//this should also promote boolean to integer, so no boolean not here...

        promoted.value = match promoted.value {
            
            LiteralValue::INTEGER(x) => LiteralValue::INTEGER(!x),
            LiteralValue::FLOAT {..} => panic!("bitwise not is not applicable for floats")
        };

        promoted.limit_literal()
    }

    pub fn boolean_not(self) -> Self {
        let mut as_bool = self.cast(&BaseType::_BOOL);

        as_bool.value = match as_bool.value {
            LiteralValue::INTEGER(0) => LiteralValue::INTEGER(1),
            LiteralValue::INTEGER(1) => LiteralValue::INTEGER(0),
            x => panic!("invalid value for boolean: {:?}", x)
        };

        as_bool.limit_literal()
    }
}

fn cmp_i128_f64(x: i128, y: f64) -> Ordering {
    if (i128::MAX as f64) < y {
        Ordering::Less//x is definitely less than y, as y > x's domain
    } else if (i128::MIN as f64) > y {
        Ordering::Greater//x is definitely greater than y, as y < x's domain
    } else {
        (x as f64).partial_cmp(&y).unwrap()
    }
}

impl PartialEq for NumberLiteral {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Neg for NumberLiteral {
    type Output = Self;

    ///unary negation `-num` for a number.
    fn neg(self) -> Self::Output {
        let mut promoted = self.unary_promote();

        promoted.value = match (&promoted.value, &promoted.data_type) {
            (LiteralValue::INTEGER(x), BaseType::I32) => LiteralValue::INTEGER((-x) as i32 as i128),
            (LiteralValue::INTEGER(x), BaseType::U32) => LiteralValue::INTEGER((-x) as u32 as i128),
            (LiteralValue::INTEGER(x), BaseType::I64) => LiteralValue::INTEGER((-x) as i64 as i128),
            (LiteralValue::INTEGER(x), BaseType::U64) => LiteralValue::INTEGER((-x) as u64 as i128),
            _ => panic!("invalid promoted type")
        };

        promoted.limit_literal()
    }
}

impl BitOr for NumberLiteral {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        let (mut l, r) = self.binary_promote(rhs);

        l.value = match (l.value, r.value) {
            (LiteralValue::INTEGER(x), LiteralValue::INTEGER(y)) => LiteralValue::INTEGER(x | y),
            _ => panic!("bitwise or is not applicable for these data types")
        };

        l.limit_literal()
    }
}

impl BitAnd for NumberLiteral {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        let (mut l, r) = self.binary_promote(rhs);

        l.value = match (l.value, r.value) {
            (LiteralValue::INTEGER(x), LiteralValue::INTEGER(y)) => LiteralValue::INTEGER(x & y),
            _ => panic!("bitwise and is not applicable for these data types")
        };

        l.limit_literal()
    }
}

impl BitXor for NumberLiteral {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        let (mut l, r) = self.binary_promote(rhs);

        l.value = match (l.value, r.value) {
            (LiteralValue::INTEGER(x), LiteralValue::INTEGER(y)) => LiteralValue::INTEGER(x ^ y),
            _ => panic!("bitwise xor is not applicable for these data types")
        };

        l.limit_literal()
    }
}

impl Add for NumberLiteral {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let (mut l, r) = self.binary_promote(rhs);

        l.value = match (l.value, r.value) {
            (LiteralValue::INTEGER(x), LiteralValue::INTEGER(y)) => LiteralValue::INTEGER(x + y),
            _ => todo!()
        };

        l.limit_literal()
    }
}
impl Sub for NumberLiteral {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let (mut l, r) = self.binary_promote(rhs);

        l.value = match (l.value, r.value) {
            (LiteralValue::INTEGER(x), LiteralValue::INTEGER(y)) => LiteralValue::INTEGER(x - y),
            _ => todo!()
        };

        l.limit_literal()
    }
}

impl Mul for NumberLiteral {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let (mut l, r) = self.binary_promote(rhs);

        l.value = match (l.value, r.value) {
            (LiteralValue::INTEGER(x), LiteralValue::INTEGER(y)) => LiteralValue::INTEGER(x * y),
            _ => todo!()
        };

        l.limit_literal()
    }
}

impl Div for NumberLiteral {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let (mut l, r) = self.binary_promote(rhs);//this should truncate self and rhs, so division should work properly...

        l.value = match (l.value, r.value) {
            (LiteralValue::INTEGER(x), LiteralValue::INTEGER(y)) => LiteralValue::INTEGER(x / y),
            _ => todo!()
        };

        l.limit_literal()
    }
}

impl Rem for NumberLiteral {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        let (mut l, r) = self.binary_promote(rhs);//this should truncate self and rhs, so division should work properly...

        l.value = match (l.value, r.value) {
            (LiteralValue::INTEGER(x), LiteralValue::INTEGER(y)) => LiteralValue::INTEGER(x % y),
            _ => panic!("invalid literal value combination for MOD")
        };

        l.limit_literal()
    }
}

impl Shl for NumberLiteral {
    type Output = Self;

    fn shl(self, rhs: Self) -> Self::Output {
        let (mut l, r) = self.binary_promote(rhs);//this should truncate self and rhs, so division should work properly...

        l.value = match (l.value, r.value) {
            (LiteralValue::INTEGER(x), LiteralValue::INTEGER(y)) => LiteralValue::INTEGER(x << y),
            _ => panic!("shl is not valid for these data types")
        };

        l.limit_literal()
    }
}

impl Shr for NumberLiteral {
    type Output = Self;
    
    fn shr(self, rhs: Self) -> Self::Output {
        let (mut l, r) = self.binary_promote(rhs);//this should truncate self and rhs, so division should work properly...

        l.value = match (l.value, r.value) {
            (LiteralValue::INTEGER(x), LiteralValue::INTEGER(y)) => LiteralValue::INTEGER(match l.data_type {
                BaseType::I8 => (x as i8 >> y) as i128,
                BaseType::U8 => (x as u8 >> y) as i128,
                BaseType::I16 => (x as i16 >> y) as i128,
                BaseType::U16 => (x as u16 >> y) as i128,
                BaseType::I32 => (x as i32 >> y) as i128,
                BaseType::U32 => (x as u32 >> y) as i128,
                BaseType::I64 => (x as i64 >> y) as i128,
                BaseType::U64 => (x as u64 >> y) as i128,
                _ => panic!("cannot shift this value"),
            }),
            _ => panic!("shl is not valid for these data types")
        };

        l.limit_literal()
    }

    
}

impl Display for NumberLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}",
        match self.value {
            LiteralValue::INTEGER(x) => x.to_string().cyan().to_string(),
            LiteralValue::FLOAT {value, ..} => format!("{:.1}", value).cyan().to_string()
        })
    }
}

impl From<i64> for NumberLiteral {
    fn from(value: i64) -> Self {
        Self { value: LiteralValue::INTEGER(value.into()), data_type: BaseType::I64 }
    }
}

impl From<String> for NumberLiteral {
    fn from(value: String) -> Self {
        Self::from(value.as_ref())
    }
}


impl From<&str> for NumberLiteral {
    //TODO maybe impl From/TryFrom???
    fn from(input: &str) -> NumberLiteral {

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

            [x @.., 'f'] if base != 16 => (Some(BaseType::F32), x),

            [x @.., 'l'] |
            x if !is_integer => (Some(BaseType::F64), x),

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
    
            println!("{:?}", forced_type);
            let data_type = forced_type.unwrap_or(predicted_type);
            assert!(data_type.is_integer());
    
            NumberLiteral {
                value:LiteralValue::INTEGER(as_large_unsigned as i128),//integer literals from text are always positive
                data_type: BaseType::U64,//start as large type, and cast from there
            }.cast(&data_type)//cast to the correct type
        } else {
            todo!("float literals")
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
/*fn calculate_float(base: i32, digits: &[char]) -> f64 {
    assert!(digits.contains(&'.'));
    todo!();
}*/