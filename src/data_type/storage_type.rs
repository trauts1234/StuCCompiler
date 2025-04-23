
/// represents how a variable is stored
pub enum StorageDuration {
    /// simple local variable stored on the stack. "register" defaults to this too
    Automatic,
    /// external storage - the value is stored in a different assembly file
    Extern,
    /// global storage - the value is associated with a label put in some sort of data section
    Static,
}