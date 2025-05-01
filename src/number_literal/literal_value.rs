use std::num::TryFromIntError;

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    INTEGER(i128)
}

impl TryInto<u64> for &LiteralValue {
    type Error = TryFromIntError;

    fn try_into(self) -> Result<u64, Self::Error> {
        match self {
            LiteralValue::INTEGER(x) => (*x).try_into()
        }
    }
}