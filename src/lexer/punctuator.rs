use crate::assembly::operation::{AsmBooleanOperation, AsmComparison};

#[derive(Debug, Clone, PartialEq)]
pub enum Punctuator {
    PLUS,
    PLUSPLUS,

    DASH,
    
    ASTERISK,
    FORWARDSLASH,
    EQUALS,
    SEMICOLON,

    PIPEPIPE,
    ANDAND,

    AMPERSAND,
    PERCENT,

    ANGLERIGHT,
    ANGLELEFT,
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

            "*" => Some(Self::ASTERISK),
            "/" => Some(Self::FORWARDSLASH),
            "=" => Some(Self::EQUALS),
            ";" => Some(Self::SEMICOLON),

            "&" => Some(Self::AMPERSAND),
            "%" => Some(Self::PERCENT),

            "||" => Some(Self::PIPEPIPE),
            "&&" => Some(Self::ANDAND),

            ">" => Some(Self::ANGLERIGHT),
            "<" => Some(Self::ANGLELEFT),
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
            Self::ANGLELEFT => Some(AsmComparison::L),
            Self::ANGLERIGHT => Some(AsmComparison::G),
            Self::DOUBLEEQUALS => Some(AsmComparison::EQ),
            Self::EXCLAMATIONEQUALS => Some(AsmComparison::NE),
            Self::LESSEQUAL => Some(AsmComparison::LE),
            Self::GREATEREQUAL => Some(AsmComparison::GE),
            _ => None,
        }
    }

    pub fn as_boolean_instr(&self) -> Option<AsmBooleanOperation> {
        match self {
            Self::PIPEPIPE => Some(AsmBooleanOperation::OR),
            Self::ANDAND => Some(AsmBooleanOperation::AND),
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

            Self::ANDAND => Some(11),
            Self::PIPEPIPE => Some(12),

            Self::EQUALS => Some(14),

            Self::ANGLELEFT | Self::ANGLERIGHT | Self::GREATEREQUAL | Self::LESSEQUAL => Some(6),
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

            Self::PLUSPLUS => Some(2),

            Self::DASH => Some(2),//unary negate
            _ => None
        }
    }
}
