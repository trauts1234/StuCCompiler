
/// represents the visibility of a globally allocated (has a label associated with it) variable
#[derive(Debug)]
pub enum GlobalAllocationType {
    /// any variable marked "extern" has this modifier, and will be imported by the linker
    Extern,
    /// any non-static global variable, as it will be exported for other translation units
    GlobalAndExport,
    /// any static variables, as it will be global but not exported to other translation units
    GlobalAndStatic
}