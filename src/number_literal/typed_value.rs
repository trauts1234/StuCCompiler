use std::{cmp::Ordering, fmt::Display, i128, ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Rem, Shl, Shr, Sub}};
use colored::Colorize;
use crate::{assembly::comparison::ComparisonKind, data_type::{base_type::{FloatType, IntegerType, ScalarType}, recursive_data_type::{calculate_promoted_type, calculate_unary_type}}, expression_visitors::expr_visitor::ExprVisitor};

#[derive(Debug, Clone)]
pub enum NumberLiteral {
    INTEGER {data: i128, data_type: IntegerType},
    FLOAT {data: f64, data_type: FloatType},
}

impl NumberLiteral {

    pub fn get_data_type(&self) -> ScalarType {
        match self {
            NumberLiteral::INTEGER {data_type, ..} => ScalarType::Integer(*data_type),
            NumberLiteral::FLOAT {data_type, ..} => ScalarType::Float(*data_type),
        }
    }

    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_number_literal(self)
    }

    /// promotes the value using unary promotion rules
    pub fn unary_promote(&self) -> Self {
        let base = calculate_unary_type(&self.get_data_type());
        self.cast(&base)
    }

    pub fn binary_promote(self, rhs: Self) -> (Self, Self) {
        let base = calculate_promoted_type(&self.get_data_type(), &rhs.get_data_type());

        (
            self.cast(&base),
            rhs.cast(&base)
        )
    }

    /// Generates the `x db 10` - type commands
    pub fn generate_data_definition_instruction(&self, variable_name: &str) -> String {
        match self {
            Self::INTEGER{data, data_type} => {
                //store the integer as a list of bytes
                format!("{} db {}",
                    variable_name,

                    data.to_le_bytes()[..data_type.memory_size().size_bytes() as usize].iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
                )
            },
            
            Self::FLOAT{data_type, data} => {
                match data_type {
                    FloatType::F32 => format!("{} dd {}", variable_name, (*data as f32).to_bits()),
                    FloatType::F64 => format!("{} dq {}", variable_name, data.to_bits()),
                }
            }
        }


    }

    pub fn cast(&self, new_type: &ScalarType) -> NumberLiteral {
        match (self, new_type) {
            (NumberLiteral::INTEGER { data, .. }, ScalarType::Integer(int_ty)) => {
                match int_ty {
                    IntegerType::I8 => NumberLiteral::INTEGER { data: *data as i8 as i128, data_type: *int_ty },
                    IntegerType::I16 => NumberLiteral::INTEGER { data: *data as i16 as i128, data_type: *int_ty },
                    IntegerType::I32 => NumberLiteral::INTEGER { data: *data as i32 as i128, data_type: *int_ty },
                    IntegerType::I64 => NumberLiteral::INTEGER { data: *data as i64 as i128, data_type: *int_ty },
                    IntegerType::U8 => NumberLiteral::INTEGER { data: *data as u8 as i128, data_type: *int_ty },
                    IntegerType::U16 => NumberLiteral::INTEGER { data: *data as u16 as i128, data_type: *int_ty },
                    IntegerType::U32 => NumberLiteral::INTEGER { data: *data as u32 as i128, data_type: *int_ty },
                    IntegerType::U64 => NumberLiteral::INTEGER { data: *data as u64 as i128, data_type: *int_ty },
                    IntegerType::_BOOL => NumberLiteral::INTEGER { data: if *data == 0 { 0 } else { 1 }, data_type: *int_ty },
                }
            }
            (NumberLiteral::INTEGER { data, .. }, ScalarType::Float(float_ty)) => {
                match float_ty {
                    FloatType::F32 => NumberLiteral::FLOAT { data: *data as f32 as f64, data_type: *float_ty },
                    FloatType::F64 => NumberLiteral::FLOAT { data: *data as f64, data_type: *float_ty },
                }
            }
            (NumberLiteral::FLOAT { data, .. }, ScalarType::Integer(int_ty)) => {
                match int_ty {
                    IntegerType::I8 => NumberLiteral::INTEGER { data: *data as i8 as i128, data_type: *int_ty },
                    IntegerType::I16 => NumberLiteral::INTEGER { data: *data as i16 as i128, data_type: *int_ty },
                    IntegerType::I32 => NumberLiteral::INTEGER { data: *data as i32 as i128, data_type: *int_ty },
                    IntegerType::I64 => NumberLiteral::INTEGER { data: *data as i64 as i128, data_type: *int_ty },
                    IntegerType::U8 => NumberLiteral::INTEGER { data: *data as u8 as i128, data_type: *int_ty },
                    IntegerType::U16 => NumberLiteral::INTEGER { data: *data as u16 as i128, data_type: *int_ty },
                    IntegerType::U32 => NumberLiteral::INTEGER { data: *data as u32 as i128, data_type: *int_ty },
                    IntegerType::U64 => NumberLiteral::INTEGER { data: *data as u64 as i128, data_type: *int_ty },
                    IntegerType::_BOOL => NumberLiteral::INTEGER { data: if *data == 0.0 { 0 } else { 1 }, data_type: *int_ty },
                }
            }
            (NumberLiteral::FLOAT { data, .. }, ScalarType::Float(float_ty)) => {
                match float_ty {
                    FloatType::F32 => NumberLiteral::FLOAT { data: *data as f32 as f64, data_type: *float_ty },
                    FloatType::F64 => NumberLiteral::FLOAT { data: *data, data_type: *float_ty },
                }
            }
        }
    }

    ///ensures that the number stored is a valid number for the data type
    pub fn limit_literal(self) -> Self {
        self.cast(&self.get_data_type())
    }

    /// promotes the number literal by applying unary plus to it
    pub fn unary_plus(self) -> Self {
        self.unary_promote()
    }

    pub fn boolean_or(self, other: Self) -> Self {
        match Self::binary_promote(self, other) {
            (NumberLiteral::INTEGER{ data:0, ..}, NumberLiteral::INTEGER{data: 0, ..}) => {
                Self::INTEGER { data: 0, data_type: IntegerType::_BOOL }// 0 | 0 = 0
            }
            (NumberLiteral::INTEGER {..}, NumberLiteral::INTEGER {..}) => {
                Self::INTEGER { data: 1, data_type: IntegerType::_BOOL }
            }

            _ => panic!("unsupported operands to boolean or")
        }
    }

    pub fn boolean_and(self, other: Self) -> Self {
        match Self::binary_promote(self, other) {
            (NumberLiteral::INTEGER{ data:x, data_type: l_type}, NumberLiteral::INTEGER{data: y, data_type: r_type})
            if x != 0 && y != 0 => {
                assert_eq!(l_type, r_type);
                Self::INTEGER { data: 1, data_type: l_type }// nonzero & nonzero = 1
            }
            (NumberLiteral::INTEGER {data_type:l_type,..}, NumberLiteral::INTEGER {data_type:r_type,..}) => {
                assert_eq!(l_type, r_type);
                Self::INTEGER { data: 0, data_type: l_type }
            }

            _ => panic!("unsupported operands to boolean or")
        }
    }

    pub fn cmp(self, other: Self, comparison: &ComparisonKind) -> bool {
        let (lhs, rhs) = self.binary_promote(other);

        let comparison_value = match (lhs, rhs) {
            (NumberLiteral::INTEGER{data: x,..}, NumberLiteral::INTEGER{data: y,..}) => x.cmp(&y),
            (NumberLiteral::INTEGER{data: x,..}, NumberLiteral::FLOAT{data: y,..}) => cmp_i128_f64(x, y),
            (NumberLiteral::FLOAT{data: x,..}, NumberLiteral::INTEGER{data: y,..}) => cmp_i128_f64(y, x).reverse(),
            (NumberLiteral::FLOAT{data: x,..}, NumberLiteral::FLOAT{data: y,..}) => x.partial_cmp(&y).unwrap(),
        };

        let cmp_result = match comparison {
            ComparisonKind::EQ => comparison_value.is_eq(),
            ComparisonKind::NE => comparison_value.is_ne(),
            ComparisonKind::L => comparison_value.is_lt(),
            ComparisonKind::LE => comparison_value.is_le(),
            ComparisonKind::G => comparison_value.is_gt(),
            ComparisonKind::GE => comparison_value.is_ge(),
            ComparisonKind::ALWAYS => panic!("invalid comparison")
        };

        cmp_result
    }

    pub fn bitwise_not(self) -> Self {
        match self.unary_promote() {
            NumberLiteral::INTEGER { data, data_type } => {
                NumberLiteral::INTEGER { data: !data, data_type }
            },
            _ => panic!("cannot bitwise not this type"),
        }.limit_literal()
    }

    pub fn boolean_not(self) -> Self {
        // a == 0 is the same operation as !a
        Self::INTEGER {
            data: self.cmp(NumberLiteral::INTEGER { data: 0, data_type: IntegerType::I64 }, &ComparisonKind::EQ) as i128,
            data_type: IntegerType::_BOOL
        }
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

impl TryInto<i128> for NumberLiteral {
    type Error = ();

    fn try_into(self) -> Result<i128, Self::Error> {
        match self {
            NumberLiteral::INTEGER { data, ..} => Ok(data),
            NumberLiteral::FLOAT {..} => Err(()),
        }
    }
}

impl PartialEq for NumberLiteral {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (NumberLiteral::INTEGER {data: l,..}, NumberLiteral::INTEGER {data:r,..}) if l == r => true,
            (NumberLiteral::FLOAT {data: l,..}, NumberLiteral::FLOAT {data:r,..}) if l == r => true,
            _ => false

        }
    }
}

impl Neg for NumberLiteral {
    type Output = Self;

    ///unary negation `-num` for a number.
    fn neg(self) -> Self::Output {
        match self.unary_promote() {
            Self::INTEGER { data, data_type } => Self::INTEGER { data: -data, data_type },
            Self::FLOAT { data, data_type } => Self::FLOAT { data: -data, data_type},
        }.limit_literal()
    }
}

impl BitOr for NumberLiteral {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        match self.binary_promote(rhs) {
            (NumberLiteral::INTEGER { data: l, data_type: l_type }, NumberLiteral::INTEGER { data: r, data_type: r_type }) => {
                assert_eq!(l_type, r_type);
                NumberLiteral::INTEGER { data: l | r, data_type: l_type }
            }
            _ => panic!("invalid operands for bitwise or")
        }
    }
}

impl BitAnd for NumberLiteral {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        match self.binary_promote(rhs) {
            (NumberLiteral::INTEGER { data: l, data_type: l_type }, NumberLiteral::INTEGER { data: r, data_type: r_type }) => {
                assert_eq!(l_type, r_type);
                NumberLiteral::INTEGER { data: l & r, data_type: l_type }
            }
            _ => panic!("invalid operands for bitwise and")
        }
    }
}

impl BitXor for NumberLiteral {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        match self.binary_promote(rhs) {
            (NumberLiteral::INTEGER { data: l, data_type: l_type }, NumberLiteral::INTEGER { data: r, data_type: r_type }) => {
                assert_eq!(l_type, r_type);
                NumberLiteral::INTEGER { data: l ^ r, data_type: l_type }
            }
            _ => panic!("invalid operands for bitwise xor")
        }
    }
}

impl Add for NumberLiteral {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match self.binary_promote(rhs) {
            (NumberLiteral::INTEGER { data: l, data_type: l_type }, NumberLiteral::INTEGER { data: r, data_type: r_type }) => {
                assert_eq!(l_type, r_type);
                NumberLiteral::INTEGER { data: l + r, data_type: l_type }.limit_literal()
            }
            (NumberLiteral::FLOAT { data: l, data_type: l_type }, NumberLiteral::FLOAT { data: r, data_type: r_type }) => {
                assert_eq!(l_type, r_type);
                NumberLiteral::FLOAT { data: l + r, data_type: l_type }.limit_literal()
            }
            _ => panic!("invalid operands for add")
        }
    }
}
impl Sub for NumberLiteral {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match self.binary_promote(rhs) {
            (NumberLiteral::INTEGER { data: l, data_type: l_type }, NumberLiteral::INTEGER { data: r, data_type: r_type }) => {
                assert_eq!(l_type, r_type);
                NumberLiteral::INTEGER { data: l - r, data_type: l_type }.limit_literal()
            }
            (NumberLiteral::FLOAT { data: l, data_type: l_type }, NumberLiteral::FLOAT { data: r, data_type: r_type }) => {
                assert_eq!(l_type, r_type);
                NumberLiteral::FLOAT { data: l - r, data_type: l_type }.limit_literal()
            }
            _ => panic!("invalid operands for subtract")
        }
    }
}

impl Mul for NumberLiteral {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match self.binary_promote(rhs) {
            (NumberLiteral::INTEGER { data: l, data_type: l_type }, NumberLiteral::INTEGER { data: r, data_type: r_type }) => {
                assert_eq!(l_type, r_type);
                NumberLiteral::INTEGER { data: l * r, data_type: l_type }.limit_literal()
            }
            (NumberLiteral::FLOAT { data: l, data_type: l_type }, NumberLiteral::FLOAT { data: r, data_type: r_type }) => {
                assert_eq!(l_type, r_type);
                NumberLiteral::FLOAT { data: l * r, data_type: l_type }.limit_literal()
            }
            _ => panic!("invalid operands for multiply")
        }
    }
}

impl Div for NumberLiteral {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match self.binary_promote(rhs) {
            (NumberLiteral::INTEGER { data: l, data_type: l_type }, NumberLiteral::INTEGER { data: r, data_type: r_type }) => {
                assert_eq!(l_type, r_type);
                NumberLiteral::INTEGER { data: l / r, data_type: l_type }.limit_literal()
            }
            (NumberLiteral::FLOAT { data: l, data_type: l_type }, NumberLiteral::FLOAT { data: r, data_type: r_type }) => {
                assert_eq!(l_type, r_type);
                NumberLiteral::FLOAT { data: l / r, data_type: l_type }.limit_literal()
            }
            _ => panic!("invalid operands for divide")
        }
    }
}

impl Rem for NumberLiteral {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        match self.binary_promote(rhs) {
            (NumberLiteral::INTEGER { data: l, data_type: l_type }, NumberLiteral::INTEGER { data: r, data_type: r_type }) => {
                assert_eq!(l_type, r_type);
                NumberLiteral::INTEGER { data: l % r, data_type: l_type }
            }
            _ => panic!("invalid operands for mod")
        }
    }
}

impl Shl for NumberLiteral {
    type Output = Self;

    fn shl(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (NumberLiteral::INTEGER { data: l, data_type: l_type }, NumberLiteral::INTEGER { data: r, ..}) => {
                NumberLiteral::INTEGER { data: l << r, data_type: l_type }.limit_literal()
            }
            _ => panic!("invalid operands for shl")
        }
    }
}

impl Shr for NumberLiteral {
    type Output = Self;
    
    fn shr(self, rhs: Self) -> Self::Output {
        
        match (self, rhs) {
            (NumberLiteral::INTEGER { data: l, data_type: l_type }, NumberLiteral::INTEGER { data: r, ..}) => {
                NumberLiteral::INTEGER {
                    data: match l_type {
                        IntegerType::I8  => (l as i8 >> r) as i128,
                        IntegerType::U8  => (l as u8 >> r) as i128,
                        IntegerType::I16 => (l as i16 >> r) as i128,
                        IntegerType::U16 => (l as u16 >> r) as i128,
                        IntegerType::I32 => (l as i32 >> r) as i128,
                        IntegerType::U32 => (l as u32 >> r) as i128,
                        IntegerType::I64 => (l as i64 >> r) as i128,
                        IntegerType::U64 => (l as u64 >> r) as i128,
                        _ => panic!("cannot shift this value"),
                    },
                    data_type: l_type
                }
            }
            _ => panic!("invalid operands for shl")
        }
    }

    
}

impl Display for NumberLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}",
        match self {
            Self::INTEGER{data,..} => data.to_string().cyan().to_string(),
            Self::FLOAT{data,..} => format!("{:.10}", data).cyan().to_string()
        })
    }
}

impl From<i64> for NumberLiteral {
    fn from(value: i64) -> Self {
        Self::INTEGER { data: value as i128, data_type: IntegerType::I64 }
    }
}

impl From<String> for NumberLiteral {
    fn from(value: String) -> Self {
        Self::from(value.as_ref())
    }
}


impl From<&str> for NumberLiteral {
    fn from(input: &str) -> NumberLiteral {
        match input.to_ascii_lowercase().chars().collect::<Vec<_>>().as_slice() {
            ['0','x', rem @ ..] => {
                let hex_data = hex_parse::hex_parse(rem);

                let has_exponent = !hex_data.exponent_part.is_empty();
                let has_fraction = !hex_data.fractional_part.is_empty();

                let integer_part = integer_value(hex_data.integer_part, 16);
                let fractional_part = fractional_value(hex_data.fractional_part, 16);
                
                if has_exponent || has_fraction || hex_data.remainder.contains(&'f') {
                    //must be a float?
                    let power: i32 = (if hex_data.negative_exponent { -1 } else { 1 } * integer_value(hex_data.exponent_part, 10)).try_into().unwrap();

                    NumberLiteral::FLOAT{
                        data: (integer_part as f64 + fractional_part) * 2f64.powi(power),
                        data_type: calculate_float_suffix_type(hex_data.remainder),
                    }.limit_literal()
                } else {
                    let data_type = calculate_int_suffix_type(hex_data.remainder);
                    NumberLiteral::INTEGER {
                        data: integer_part,
                        data_type: data_type.unwrap_or(calculate_integer_type(integer_part))
                    }.limit_literal()
                }
            },

            ['0', 'b', rem @ ..] => {
                let bin_data = bin_parse::bin_parse(rem);
                let integer_part = integer_value(bin_data.integer_part, 2);
                let data_type = calculate_int_suffix_type(bin_data.remainder);

                NumberLiteral::INTEGER {
                    data: integer_part,
                    data_type: data_type.unwrap_or(calculate_integer_type(integer_part))
                }.limit_literal()
            },

            ['0', rem @ ..] if rem.iter().all(|c| matches!(c, '0'..='7' | 'u' | 'l')) => {
                let oct_data = oct_parse::oct_parse(rem);
                let integer_part = integer_value(oct_data.integer_part, 8);
                let data_type = calculate_int_suffix_type(oct_data.remainder);

                NumberLiteral::INTEGER {
                    data: integer_part,
                    data_type: data_type.unwrap_or(calculate_integer_type(integer_part))
                }.limit_literal()
            },

            rem => {
                let dec_data = dec_parse::dec_parse(rem);

                let has_exponent = !dec_data.exponent_part.is_empty();
                let has_fraction = !dec_data.decimal_part.is_empty();

                let integer_part = integer_value(dec_data.integer_part, 10);
                let fractional_part = fractional_value(dec_data.decimal_part, 10);
                
                if has_exponent || has_fraction || dec_data.remainder.contains(&'f') {
                    //must be a float?
                    let power: i32 = (if dec_data.negative_exponent { -1 } else { 1 } * integer_value(dec_data.exponent_part, 10)).try_into().unwrap();

                    NumberLiteral::FLOAT{
                        data: (integer_part as f64 + fractional_part) * 10f64.powi(power),
                        data_type: calculate_float_suffix_type(dec_data.remainder),
                    }.limit_literal()
                } else {
                    let data_type = calculate_int_suffix_type(dec_data.remainder);
                    NumberLiteral::INTEGER {
                        data: integer_part,
                        data_type: data_type.unwrap_or(calculate_integer_type(integer_part))
                    }.limit_literal()
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
fn fractional_value(fractional_digits: &[char], base: u32) -> f64 {
    if fractional_digits.is_empty() {
        0f64
    } 
    else {
        fractional_digits
        .iter()
        .zip(powers_iter(base.into()).skip(1))//skip the first as the fractional part starts *after* the ones place
        .map(|(digit, multiplier)| {
            let digit_value: f64 = digit.to_digit(base).unwrap().into();
            let divisor: f64 = 1f64 / (multiplier as f64);
            digit_value * divisor// fractional value = digit * 1/base^n
        })
        .sum()
    }
}

/// Finds the best integer type to hold `value`
fn calculate_integer_type(value: i128) -> IntegerType {
    match value {
        0..=2_147_483_647 => IntegerType::I32,
        2_147_483_648..=9_223_372_036_854_775_807 => IntegerType::I64,
        9_223_372_036_854_775_808..=18446744073709551615 => IntegerType::U64,
        ..0 => panic!("negative integer literal value?"),
        _ => panic!("out of range value to be stored in C")
    }
}

fn calculate_float_suffix_type(suffix: &[char]) -> FloatType {
    match suffix {
        ['f'] => FloatType::F32,
        [] => FloatType::F64,
        x => panic!("invalid suffix for float literal: {:?}", x)
    }
}
/// This is fallible because no suffix -> type calculation
fn calculate_int_suffix_type(suffix: &[char]) -> Option<IntegerType> {
    match suffix {
        ['u','l','l'] | //ull
        ['l', 'l', 'u'] |//llu
        ['u','l'] |//ul
        ['l', 'u'] //lu
            => Some(IntegerType::U64),//all are u64

        ['u'] => Some(IntegerType::U32),//u

        ['l', 'l' ] | //ll
        ['l'] //l
            => Some(IntegerType::I64),//are i64
        
        [] => None,//no suffix, predict type based on data size

        x => panic!("invalid suffix for integer literal: {:?}", x)
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