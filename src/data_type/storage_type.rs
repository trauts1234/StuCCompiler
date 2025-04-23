
/// represents how a variable is stored
pub enum StorageDuration {
    /// stored in the default location, either file scope(globally), or local scope. "register" defaults to this
    Default,
    /// external storage - the value is stored in a different assembly file
    Extern,
    /// global storage - the value is associated with a label put in some sort of data section
    Static,
}