use std::fmt::Display;
use std::hash::Hash;
use unwrap_let::unwrap_let;
use uuid::Uuid;

use crate::data_type::base_type::IntegerType;
use crate::data_type::recursive_data_type::DataType;
use crate::expression::expression::Expression;
use crate::expression_visitors::expr_visitor::ExprVisitor;
use crate::number_literal::typed_value::NumberLiteral;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringLiteral {
    text: Vec<i8>,//text plus zero terminator
    label: String
}

impl StringLiteral {

    /// Includes the zero terminator
    pub fn get_num_chars(&self) -> usize {
        self.text.len()
    }

    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_string_literal(self)
    }
}

impl StringLiteral {
    pub fn get_label(&self) -> &str {
        &self.label
    }
    pub fn get_comma_separated_bytes(&self) -> String {
        self.text.iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(",")
    }
    pub fn try_new(inside_speechmarks: &str) -> Option<StringLiteral> {
        assert!(inside_speechmarks.is_ascii());
    
        Some(StringLiteral {
            label: format!("string_{}", Uuid::new_v4().simple()),
            text: Self::use_escape_sequences(inside_speechmarks)
        })
    }
    pub fn new_from_raw(inside_speechmarks: impl Iterator<Item=char>) -> Self {
        let mut text: Vec<i8> = inside_speechmarks.map(|x| x as i8).collect();
        if !text.ends_with(&[0]) {
            text.push(0);//add null terminator
        }
        StringLiteral {
            label: format!("string_{}", Uuid::new_v4().simple()),
            text
        }
    }

    pub fn use_escape_sequences(text: &str) -> Vec<i8> {
        text
        .replace("\\0", "\0")//replace end of string
        .replace("\\a", "\u{07}")//replace bell
        .replace("\\b", "\u{08}")//replace backspace
        .replace("\\f", "\u{0C}")//replace end page (form feed)
        .replace("\\n", "\n")//replace \n with an actual newline
        .replace("\\r", "\r")//replace \r
        .replace("\\t", "\t")//replace tab
        .replace("\\v", "\u{0B}")//replace vertical tab
        .replace("\\\\", "\\")//replace backslash (awkward because rust has to have escaped backslash too)
        .replace("\\'", "\'")//replace single quote
        .replace("\\\"", "\"")//replace double quote
        .replace("\\?", "?")//defend against trigraphs with \?
        //TODO hex escape sequences etc.
        .chars()
        .map(|x| x as i8)//convert to integers
        .chain(std::iter::once(0))//add null terminator 0
        .collect()
    }

    pub fn zero_fill_and_flatten_to_iter(&self, array_data_type: &DataType) -> Vec<Expression> {
        unwrap_let!(DataType::ARRAY{size, ..} = array_data_type);

        let extra_zeroes = size.checked_sub(self.text.len() as u64).unwrap();

        self.text.iter()
        .chain(std::iter::repeat_n(&0i8, extra_zeroes as usize))
        .map(|num| Expression::NUMBERLITERAL(NumberLiteral::INTEGER { data: (*num).into(), data_type: IntegerType::I8 }))
        .collect()
    }
}

impl Hash for StringLiteral {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.label.hash(state);
    }
}

impl Display for StringLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.text.iter().map(|x| *x as u8 as char).collect::<String>())
    }
}