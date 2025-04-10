use crate::assembly::operation::{LogicalOperation, AsmComparison};

#[derive(Debug, Clone, PartialEq)]
pub enum Punctuator {
    PLUS,
    PLUSPLUS,

    DASH,
    DASHDASH,
    
    ASTERISK,
    FORWARDSLASH,
    EQUALS,
    SEMICOLON,

    Pipe,
    PIPEPIPE,
    ANDAND,

    AMPERSAND,
    PERCENT,

    Greater,
    GreaterGreater,
    Less,
    LessLess,
    LESSEQUAL,
    GREATEREQUAL,
    DOUBLEEQUALS,
    EXCLAMATIONEQUALS,

    OPENCURLY,
    CLOSECURLY,
    OPENSQUIGGLY,
    CLOSESQUIGGLY,
    OPENSQUARE,
    CLOSESQUARE,
    COMMA,

    FULLSTOP,
    ELIPSIS,
}

impl Punctuator {
    pub fn try_new(to_token: &str) -> Option<Punctuator> {
        match to_token {
            "+" => Some(Self::PLUS),
            "++" => Some(Self::PLUSPLUS),
            
            "-" => Some(Self::DASH),
            "--" => Some(Self::DASHDASH),

            "*" => Some(Self::ASTERISK),
            "/" => Some(Self::FORWARDSLASH),
            "=" => Some(Self::EQUALS),
            ";" => Some(Self::SEMICOLON),

            "&" => Some(Self::AMPERSAND),
            "%" => Some(Self::PERCENT),

            "|" => Some(Self::Pipe),
            "||" => Some(Self::PIPEPIPE),
            "&&" => Some(Self::ANDAND),

            ">" => Some(Self::Greater),
            ">>" => Some(Self::GreaterGreater),
            "<" => Some(Self::Less),
            "<<" => Some(Self::LessLess),
            ">=" => Some(Self::GREATEREQUAL),
            "<=" => Some(Self::LESSEQUAL),
            "==" => Some(Self::DOUBLEEQUALS),
            "!=" => Some(Self::EXCLAMATIONEQUALS),

            "(" => Some(Self::OPENCURLY),
            ")" => Some(Self::CLOSECURLY),
            "{" => Some(Self::OPENSQUIGGLY),
            "}" => Some(Self::CLOSESQUIGGLY),
            "[" => Some(Self::OPENSQUARE),
            "]" => Some(Self::CLOSESQUARE),

            "," => Some(Self::COMMA),

            "." => Some(Self::FULLSTOP),
            "..." => Some(Self::ELIPSIS),
            _ => None
        }
    }

    /**
     * if this punctuator is a comparison operator, what instruction would
     * returns the correct setcc instruction
     */
    pub fn as_comparator_instr(&self) -> Option<AsmComparison> {
        match self {
            Self::Less => Some(AsmComparison::L),
            Self::Greater => Some(AsmComparison::G),
            Self::DOUBLEEQUALS => Some(AsmComparison::EQ),
            Self::EXCLAMATIONEQUALS => Some(AsmComparison::NE),
            Self::LESSEQUAL => Some(AsmComparison::LE),
            Self::GREATEREQUAL => Some(AsmComparison::GE),
            _ => None,
        }
    }

    pub fn as_boolean_instr(&self) -> Option<LogicalOperation> {
        match self {
            Self::PIPEPIPE => Some(LogicalOperation::OR),
            Self::ANDAND => Some(LogicalOperation::AND),
            _ => None
        }
    }

    pub fn as_bitwise_instr(&self) -> Option<LogicalOperation> {
        match self {
            Self::Pipe => Some(LogicalOperation::OR),

            _ => None
        }
    }

     /**
     * if this punctuator can be a binary operator:
     * returns Some(precedence number)
     * if it can't: None
     */
    pub fn as_binary_operator_precedence(&self) -> Option<i32> {

        match self {
            Self::PLUS | Self::DASH => Some(4),
            Self::ASTERISK | Self::FORWARDSLASH | Self::PERCENT => Some(3),//binary operator as in multiply

            Self::LessLess | Self::GreaterGreater => Some(5),//bitwise shifts

            Self::Pipe => Some(10),//bitwise or

            Self::ANDAND => Some(11),
            Self::PIPEPIPE => Some(12),

            Self::EQUALS => Some(14),

            Self::Less | Self::Greater | Self::GREATEREQUAL | Self::LESSEQUAL => Some(6),
            Self::DOUBLEEQUALS | Self::EXCLAMATIONEQUALS => Some(7),
            _ => None
        }
    }
    /**
     * if this punctuator can be a unary prefix operator:
     * returns Some(precedence number)
     * if it can't: None
     */
    pub fn as_unary_prefix_precedence(&self) -> Option<i32> {
        match self {
            Self::ASTERISK => Some(2),//dereference
            Self::AMPERSAND => Some(2),//reference

            Self::PLUSPLUS | Self::DASHDASH => Some(2),//prefix increment/decrement

            Self::DASH => Some(2),//unary negate
            _ => None
        }
    }
}
