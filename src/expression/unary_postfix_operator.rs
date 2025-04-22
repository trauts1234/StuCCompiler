use crate::lexer::punctuator::Punctuator;

#[derive(Clone, Debug, PartialEq)]
pub enum UnaryPostfixOperator {
    Decrement,
    Increment,
}

impl TryFrom<Punctuator> for UnaryPostfixOperator {
    type Error = ();

    fn try_from(value: Punctuator) -> Result<Self, Self::Error> {
        match value {
            Punctuator::DASHDASH => Ok(Self::Decrement),
            Punctuator::PLUSPLUS => Ok(Self::Increment),
            _ => Err(())
        }
    }
}

impl<'a> Into<&'a str> for UnaryPostfixOperator {
    fn into(self) -> &'a str {
        match self {
            Self::Decrement => "decrement",
            Self::Increment => "increment",
        }
    }
}