use crate::lexer::punctuator::Punctuator;

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