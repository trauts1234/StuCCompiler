use std::{cmp::Ordering, fmt::Display, i128, ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Rem, Shl, Shr, Sub}};
use colored::Colorize;
use unwrap_let::unwrap_let;
use uuid::Uuid;
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
            
            LiteralValue::FLOAT(value) => {
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
            (LiteralValue::FLOAT (value), BaseType::_BOOL) => LiteralValue::INTEGER(if value == 0.0 {0} else {1}),

            (LiteralValue::FLOAT(value), BaseType::F32) => LiteralValue::FLOAT(value as f32 as f64),
            (LiteralValue::FLOAT(value), BaseType::F64) => LiteralValue::FLOAT(value),//already a f64

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
            (LiteralValue::INTEGER(x), LiteralValue::FLOAT(y)) => cmp_i128_f64(x, y),
            (LiteralValue::FLOAT(x), LiteralValue::INTEGER(y)) => cmp_i128_f64(y, x).reverse(),
            (LiteralValue::FLOAT(x), LiteralValue::FLOAT(y)) => x.partial_cmp(&y).unwrap(),
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

        promoted.value = match (promoted.value.clone(), &promoted.data_type) {
            (LiteralValue::INTEGER(x), BaseType::I32) => LiteralValue::INTEGER((-x) as i32 as i128),
            (LiteralValue::INTEGER(x), BaseType::U32) => LiteralValue::INTEGER((-x) as u32 as i128),
            (LiteralValue::INTEGER(x), BaseType::I64) => LiteralValue::INTEGER((-x) as i64 as i128),
            (LiteralValue::INTEGER(x), BaseType::U64) => LiteralValue::INTEGER((-x) as u64 as i128),
            (LiteralValue::FLOAT(value), BaseType::F32) => LiteralValue::FLOAT((-value) as f32 as f64),
            (LiteralValue::FLOAT(value), BaseType::F64) => LiteralValue::FLOAT(-value),
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
            LiteralValue::FLOAT(value) => format!("{:.1}", value).cyan().to_string()
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
        match input.to_ascii_lowercase().chars().collect::<Vec<_>>().as_slice() {
            ['0','x', rem @ ..] => {
                let hex_data = hex_parse::hex_parse(rem);
                let integer_part = integer_value(hex_data.integer_part, 16);
                let fractional_part = fractional_value(hex_data.fractional_part, 16);
                let power: i32 = (if hex_data.negative_exponent { -1 } else { 1 } * integer_value(hex_data.exponent_part, 10)).try_into().unwrap();
                
                if let Some(frac) = fractional_part {
                    NumberLiteral {
                        value: LiteralValue::FLOAT((integer_part as f64 + frac) * 2f64.powi(power)),
                        data_type: calculate_suffix_type(hex_data.remainder, true).unwrap_or(BaseType::F64),
                    }
                } else {
                    NumberLiteral {
                        value: LiteralValue::INTEGER(integer_part * 2i128.pow(power.try_into().unwrap())),//just an integer
                        //read the suffix (non-float) or calculate the best type
                        data_type: calculate_suffix_type(hex_data.remainder, false).unwrap_or(calculate_integer_type(integer_part)),
                    }
                }
            },

            ['0', 'b', rem @ ..] => {
                let bin_data = bin_parse::bin_parse(rem);
                let integer_part = integer_value(bin_data.integer_part, 2);

                NumberLiteral {
                    value: LiteralValue::INTEGER(integer_part),
                    data_type: calculate_suffix_type(bin_data.remainder, false).unwrap_or(calculate_integer_type(integer_part)),
                }
            },

            ['0', rem @ ..] => {
                let oct_data = oct_parse::oct_parse(rem);
                let integer_part = integer_value(oct_data.integer_part, 8);

                NumberLiteral {
                    value: LiteralValue::INTEGER(integer_part),
                    data_type: calculate_suffix_type(oct_data.remainder, false).unwrap_or(calculate_integer_type(integer_part)),
                }
            },

            rem => {
                let dec_data = dec_parse::dec_parse(rem);
                let integer_part = integer_value(dec_data.integer_part, 10);
                let fractional_part = fractional_value(dec_data.decimal_part, 10);
                let suffix_type = calculate_suffix_type(dec_data.remainder, fractional_part.is_some());
                let power: i32 = (if dec_data.negative_exponent { -1 } else { 1 } * integer_value(dec_data.exponent_part, 10)).try_into().unwrap();
                
                if let Some(frac) = fractional_part {
                    NumberLiteral {
                        value: LiteralValue::FLOAT((integer_part as f64 + frac) * 10f64.powi(power)),
                        data_type: suffix_type.unwrap_or(BaseType::F64),
                    }
                } else {
                    NumberLiteral {
                        value: LiteralValue::INTEGER(integer_part * 10i128.pow(power.try_into().unwrap())),//just an integer
                        //read the suffix (non-float) or calculate the best type
                        data_type: suffix_type.unwrap_or(calculate_integer_type(integer_part)),
                    }
                }
            }
        }

    }

}

fn powers_iter(base: i128) -> impl Iterator<Item = i128> {
    std::iter::successors(Some(1), move |&k| Some(k * base))
}
fn integer_value(integer_digits: &[char], base: u32) -> i128 {
    integer_digits
    .iter()
    .rev()//start with least significant digit
    .map(|digit| digit.to_digit(base).unwrap().into())//get digit value
    .zip(powers_iter(base.into()))//add powers of base
    .map(|(digit, multiplier): (i128, i128)| multiplier * digit)//get digit value and multiply by power
    .sum()//find total value
}

/// Returns `None` if there are no digits
fn fractional_value(fractional_digits: &[char], base: u32) -> Option<f64> {
    if fractional_digits.is_empty() {
        None
    } 
    else {Some(
        fractional_digits
        .iter()
        .zip(powers_iter(base.into()))
        .map(|(digit, multiplier)| {
            let digit_value: f64 = digit.to_digit(base).unwrap().into();
            let divisor: f64 = 1f64 / (multiplier as f64);
            digit_value/divisor// fractional value = digit * 1/base^n
        })
        .sum()
    )}
}

/// Finds the best integer type to hold `value`
fn calculate_integer_type(value: i128) -> BaseType {
    match value {
        0..=2_147_483_647 => BaseType::I32,
        2_147_483_648..=9_223_372_036_854_775_807 => BaseType::I64,
        9_223_372_036_854_775_808..=18446744073709551615 => BaseType::U64,
        ..0 => panic!("negative integer literal value?"),
        _ => panic!("out of range value to be stored in C")
    }
}
fn calculate_suffix_type(suffix: &[char], has_fractional_part: bool) -> Option<BaseType> {
    match suffix {
        ['f'] => Some(BaseType::F32),

        ['l'] |
        _ if has_fractional_part => Some(BaseType::F64),

        //integer types

        ['u','l','l'] | //ull
        ['l', 'l', 'u'] |//llu
        ['u','l'] |//ul
        ['l', 'u'] //lu
            if !has_fractional_part => Some(BaseType::U64),//all are u64

        ['u'] if !has_fractional_part => Some(BaseType::U32),//u

        ['l', 'l' ] | //ll
        ['l'] //l
            if !has_fractional_part => Some(BaseType::I64),//are i64
        
        _ if !has_fractional_part => None,//no suffix, predict type based on data size

        _ => panic!("invalid literal suffix")
    }
}

mod dec_parse {
    use crate::number_literal::typed_value::dec_run;

    pub struct DecParseResult<'a> {
        pub integer_part: &'a[char],
        pub decimal_part: &'a[char],
        pub exponent_part: &'a[char],
        pub remainder: &'a[char],
        pub negative_exponent: bool,
    }
    /// Parses the whole of a hex number including decimals and exponents
    pub fn dec_parse(remainder: &[char]) -> DecParseResult {
        println!("{:?}", remainder);
        //parse the `123` part of `123.44e10`
        let (integer_part, remainder) = dec_run(remainder);

        // parse the `44` part of `.44e10`
        let (decimal_part, remainder) = match remainder.first() {
            Some('.') => dec_run(&remainder[1..]),
            _ => (&remainder[0..0], remainder)//probably an exponent
        };

        //10e-2
        let negative_exponent = remainder.get(1).is_some_and(|tok| *tok == '-');
        let (exponent_part, remainder) = match remainder {
            ['e', '-', rem @ ..] |
            ['e', '+', rem @ ..] |
            ['e', rem @ ..] => dec_run(rem),
            _ => (&remainder[0..0], remainder)
        };

        DecParseResult { 
            integer_part,
            decimal_part,
            exponent_part,
            remainder,
            negative_exponent
        }
    }
}

mod hex_parse {
    use crate::number_literal::typed_value::{dec_run, hex_run};

    pub struct HexParseResult<'a> {
        pub integer_part: &'a[char],
        pub fractional_part: &'a[char],
        pub exponent_part: &'a[char],
        pub remainder: &'a[char],
        pub negative_exponent: bool,
    }
    /// Parses the whole of a hex number including decimals and exponents
    pub fn hex_parse(remainder: &[char]) -> HexParseResult {
        //parse the `ff` part of `ff.ep10`
        let (integer_part, remainder) = hex_run(remainder);

        // parse the `e` part of `.ep10`
        let (decimal_part, remainder) = match remainder.first() {
            Some('.') => dec_run(&remainder[1..]),
            _ => (&remainder[0..0], remainder)//probably an exponent
        };

        //10e-2
        let negative_exponent = remainder.get(1).is_some_and(|tok| *tok == '-');
        let (exponent_part, remainder) = match remainder {
            ['p', '-', rem @ ..] |
            ['p', '+', rem @ ..] |
            ['p', rem @ ..] => dec_run(rem),//hex exponents are still base 10
            _ => (&remainder[0..0], remainder)
        };

        HexParseResult { 
            integer_part,
            fractional_part: decimal_part,
            exponent_part,
            remainder,
            negative_exponent
        }
    }
}

mod bin_parse {
    use crate::number_literal::typed_value::bin_run;

    pub struct BinParseResult<'a> {
        pub integer_part: &'a[char],
        pub remainder: &'a[char],
    }
    /// Parses the whole of a binary number including decimals and exponents
    pub fn bin_parse(remainder: &[char]) -> BinParseResult {

        let (integer_part, remainder) = bin_run(remainder);

        BinParseResult { 
            integer_part,
            remainder
        }
    }
}

mod oct_parse {
    use crate::number_literal::typed_value::oct_run;

    pub struct OctParseResult<'a> {
        pub integer_part: &'a[char],
        pub remainder: &'a[char],
    }
    /// Parses the whole of an octal number including decimals and exponents
    pub fn oct_parse(remainder: &[char]) -> OctParseResult {

        let (integer_part, remainder) = oct_run(remainder);

        OctParseResult { 
            integer_part,
            remainder
        }
    }
}

/// Parses decimal digits and returns the slice and the remainder
fn dec_run(digits: &[char]) -> (&[char], &[char]) {
    let end_idx = digits
    .iter()
    .position(|c| match c {
        '0'..='9' => false,//hex digit, keep going
        _ => true
    })
    .unwrap_or(digits.len());//if all are digits, end_idx = length of digits

    (&digits[0..end_idx], &digits[end_idx..])
}

/// Parses hex digits and returns the slice and the remainder
fn hex_run(digits: &[char]) -> (&[char], &[char]) {
    let end_idx = digits
    .iter()
    .position(|c| match c {
        '0'..='9' |
        'a' ..='f' => false,//hex digit, keep going
        _ => true
    })
    .unwrap_or(digits.len());//if all are digits, end_idx = length of digits

    (&digits[0..end_idx], &digits[end_idx..])
}
/// Parses binary digits and returns the slice and the remainder
fn bin_run(digits: &[char]) -> (&[char], &[char]) {
    let end_idx = digits
    .iter()
    .position(|c| match c {
        '0' | '1' => false,//binary digit, keep going
        _ => true
    })
    .unwrap_or(digits.len());//if all are digits, end_idx = length of digits

    (&digits[0..end_idx], &digits[end_idx..])
}
/// Parsesn octal digits and returns the slice and the remainder
fn oct_run(digits: &[char]) -> (&[char], &[char]) {
    let end_idx = digits
    .iter()
    .position(|c| match c {
        '0' ..='7' => false,//an octal digit, keep going
        _ => true
    })
    .unwrap_or(digits.len());//if all are digits, end_idx = length of digits

    (&digits[0..end_idx], &digits[end_idx..])
}