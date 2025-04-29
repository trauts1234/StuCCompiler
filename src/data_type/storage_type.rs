
/// represents how a variable is stored
#[derive(Debug, Clone, PartialEq)]
pub enum StorageDuration {
    /// stored in the default location, either file scope(globally), or local scope. "register" defaults to this
    Default,
    /// external storage - the value is stored in a different assembly file
    Extern,
    /// global storage - the value is associated with a label put in some sort of data section
    Static,
}

impl<'a> TryFrom<&'a str> for StorageDuration {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "auto" => Ok(Self::Default),
            "extern" => Ok(Self::Extern),
            "static" => Ok(Self::Static),
            _ => Err(())
        }
    }
}