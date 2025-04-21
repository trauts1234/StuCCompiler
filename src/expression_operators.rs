use crate::{assembly::operation::{AsmComparison, LogicalOperation}, lexer::punctuator::Punctuator};

#[derive(Clone, Debug, PartialEq)]
pub enum UnaryPrefixOperator {
    Reference,
    Dereference,
    Negate,
    UnaryPlus,
    Decrement,
    Increment,
    BooleanNot,
    BitwiseNot
}

impl TryFrom<Punctuator> for UnaryPrefixOperator {
    type Error = ();

    fn try_from(value: Punctuator) -> Result<Self, Self::Error> {
        match value {
            Punctuator::AMPERSAND => Ok(Self::Reference),
            Punctuator::ASTERISK => Ok(Self::Dereference),
            Punctuator::DASH => Ok(Self::Negate),
            Punctuator::PLUS => Ok(Self::UnaryPlus),
            Punctuator::DASHDASH => Ok(Self::Decrement),
            Punctuator::PLUSPLUS => Ok(Self::Increment),
            Punctuator::Exclamation => Ok(Self::BooleanNot),
            Punctuator::Tilde => Ok(Self::BitwiseNot),
            _ => Err(())
        }
    }
}

impl<'a> Into<&'a str> for UnaryPrefixOperator {
    fn into(self) -> &'a str {
        match self {
            UnaryPrefixOperator::Reference => "reference",
            UnaryPrefixOperator::Dereference => "dereference",
            UnaryPrefixOperator::Negate => "negate",
            UnaryPrefixOperator::UnaryPlus => "unary plus",
            UnaryPrefixOperator::Decrement => "decrement",
            UnaryPrefixOperator::Increment => "increment",
            UnaryPrefixOperator::BooleanNot => "boolean not",
            UnaryPrefixOperator::BitwiseNot => "bitwise not",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BinaryExpressionOperator {
    Assign,

    BooleanOr,
    BooleanAnd,

    Add,
    Subtract,
    Multiply,
    Divide,
    Mod,

    CmpLess,
    CmpGreater,
    CmpLessEqual,
    CmpGreaterEqual,
    CmpEqual,
    CmpNotEqual,

    BitwiseOr,
    BitwiseAnd,
    BitwiseXor,

    BitshiftRight,
    BitshiftLeft,

}

impl BinaryExpressionOperator {
    pub fn as_boolean_instr(&self) -> Option<LogicalOperation> {
        match self {
            Self::BooleanOr => Some(LogicalOperation::OR),
            Self::BooleanAnd => Some(LogicalOperation::AND),
            _ => None
        }
    }

    /**
     * if this punctuator is a comparison operator, what instruction would
     * returns the correct setcc instruction
     */
    pub fn as_comparator_instr(&self) -> Option<AsmComparison> {
        match self {
            Self::CmpLess => Some(AsmComparison::L),
            Self::CmpGreater => Some(AsmComparison::G),
            Self::CmpEqual => Some(AsmComparison::EQ),
            Self::CmpNotEqual => Some(AsmComparison::NE),
            Self::CmpLessEqual => Some(AsmComparison::LE),
            Self::CmpGreaterEqual => Some(AsmComparison::GE),
            _ => None,
        }
    }

    pub fn as_bitwise_binary_instr(&self) -> Option<LogicalOperation> {
        match self {
            Self::BitwiseOr => Some(LogicalOperation::OR),
            Self::BitwiseAnd => Some(LogicalOperation::AND),
            Self::BitwiseXor => Some(LogicalOperation::XOR),
            _ => None
        }
    }
}

impl TryFrom<Punctuator> for BinaryExpressionOperator {
    type Error = ();

    fn try_from(value: Punctuator) -> Result<Self, Self::Error> {
        match value {
            Punctuator::EQUALS => Ok(Self::Assign),
            Punctuator::PIPEPIPE => Ok(Self::BooleanOr),
            Punctuator::ANDAND => Ok(Self::BooleanAnd),
            Punctuator::PLUS => Ok(Self::Add),
            Punctuator::DASH => Ok(Self::Subtract),
            Punctuator::ASTERISK => Ok(Self::Multiply),
            Punctuator::FORWARDSLASH => Ok(Self::Divide),
            Punctuator::PERCENT => Ok(Self::Mod),
            Punctuator::Less => Ok(Self::CmpLess),
            Punctuator::Greater => Ok(Self::CmpGreater),
            Punctuator::LESSEQUAL => Ok(Self::CmpLessEqual),
            Punctuator::GREATEREQUAL => Ok(Self::CmpGreaterEqual),
            Punctuator::DOUBLEEQUALS => Ok(Self::CmpEqual),
            Punctuator::EXCLAMATIONEQUALS => Ok(Self::CmpNotEqual),
            Punctuator::Pipe => Ok(Self::BitwiseOr),
            Punctuator::AMPERSAND => Ok(Self::BitwiseAnd),
            Punctuator::Hat => Ok(Self::BitwiseXor),
            Punctuator::GreaterGreater => Ok(Self::BitshiftRight),
            Punctuator::LessLess => Ok(Self::BitshiftLeft),

            _ => Err(()),
        }
    }
}

impl<'a> Into<&'a str> for BinaryExpressionOperator {
    fn into(self) -> &'a str {
        match self {
            BinaryExpressionOperator::Assign => "assign",
            BinaryExpressionOperator::BooleanOr => "boolean or",
            BinaryExpressionOperator::BooleanAnd => "boolean and",
            BinaryExpressionOperator::Add => "add",
            BinaryExpressionOperator::Subtract => "subtract",
            BinaryExpressionOperator::Multiply => "multiply",
            BinaryExpressionOperator::Divide => "divide",
            BinaryExpressionOperator::Mod => "mod",
            BinaryExpressionOperator::CmpLess => "compare <",
            BinaryExpressionOperator::CmpGreater => "compare >",
            BinaryExpressionOperator::CmpLessEqual => "compare <=",
            BinaryExpressionOperator::CmpGreaterEqual => "compare >=",
            BinaryExpressionOperator::CmpEqual => "compare ==",
            BinaryExpressionOperator::CmpNotEqual => "compare !=",
            BinaryExpressionOperator::BitwiseOr => "bitwise or",
            BinaryExpressionOperator::BitwiseAnd => "bitwise and",
            BinaryExpressionOperator::BitwiseXor => "bitwise xor",
            BinaryExpressionOperator::BitshiftRight => "shift right",
            BinaryExpressionOperator::BitshiftLeft => "shift left",
        }
    }
}