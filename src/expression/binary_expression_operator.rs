use crate::{assembly::{comparison::ComparisonKind, operation::LogicalOperation}, lexer::punctuator::Punctuator};

#[derive(Clone, Debug, PartialEq)]
pub enum BinaryExpressionOperator {
    Assign,
    AdditionCombination,
    SubtractionCombination,

    BooleanOr,
    BooleanAnd,

    Add,
    Subtract,
    Multiply,
    Divide,
    Mod,

    ///TODO how about a COMPARISON(ComparisonKind) variant?
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
    pub fn as_comparator_instr(&self) -> Option<ComparisonKind> {
        match self {
            Self::CmpLess => Some(ComparisonKind::L),
            Self::CmpGreater => Some(ComparisonKind::G),
            Self::CmpEqual => Some(ComparisonKind::EQ),
            Self::CmpNotEqual => Some(ComparisonKind::NE),
            Self::CmpLessEqual => Some(ComparisonKind::LE),
            Self::CmpGreaterEqual => Some(ComparisonKind::GE),
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

    pub fn from_punctuator(value: Punctuator) -> Option<Self> {
        match value {
            Punctuator::EQUALS => Some(Self::Assign),
            Punctuator::PIPEPIPE => Some(Self::BooleanOr),
            Punctuator::ANDAND => Some(Self::BooleanAnd),
            Punctuator::PLUS => Some(Self::Add),
            Punctuator::DASH => Some(Self::Subtract),
            Punctuator::ASTERISK => Some(Self::Multiply),
            Punctuator::FORWARDSLASH => Some(Self::Divide),
            Punctuator::PERCENT => Some(Self::Mod),
            Punctuator::Less => Some(Self::CmpLess),
            Punctuator::Greater => Some(Self::CmpGreater),
            Punctuator::LESSEQUAL => Some(Self::CmpLessEqual),
            Punctuator::GREATEREQUAL => Some(Self::CmpGreaterEqual),
            Punctuator::DOUBLEEQUALS => Some(Self::CmpEqual),
            Punctuator::EXCLAMATIONEQUALS => Some(Self::CmpNotEqual),
            Punctuator::Pipe => Some(Self::BitwiseOr),
            Punctuator::AMPERSAND => Some(Self::BitwiseAnd),
            Punctuator::Hat => Some(Self::BitwiseXor),
            Punctuator::GreaterGreater => Some(Self::BitshiftRight),
            Punctuator::LessLess => Some(Self::BitshiftLeft),
            Punctuator::AdditionCombination => Some(Self::AdditionCombination),
            Punctuator::SubtractionCombination => Some(Self::SubtractionCombination),

            _ => None,
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
            Self::AdditionCombination => "increment by",
            Self::SubtractionCombination => "subtract by",
        }
    }
}