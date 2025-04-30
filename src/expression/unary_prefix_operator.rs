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
    BitwiseNot,
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
            Self::Reference => "reference",
            Self::Dereference => "dereference",
            Self::Negate => "negate",
            Self::UnaryPlus => "unary plus",
            Self::Decrement => "decrement",
            Self::Increment => "increment",
            Self::BooleanNot => "boolean not",
            Self::BitwiseNot => "bitwise not",
        }
    }
}