#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    INTEGER(i128),
    /// Float has a label as it needs to be set as a constant
    FLOAT(f64)
}

impl TryInto<u64> for &LiteralValue {
    type Error = ();

    fn try_into(self) -> Result<u64, Self::Error> {
        match self {
            LiteralValue::INTEGER(x) => (*x).try_into().map_err(|_| ()),
            LiteralValue::FLOAT(_) => panic!("tried to cast float to u64 but that is difficult"),
        }
    }
}