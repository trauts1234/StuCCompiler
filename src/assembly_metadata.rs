use crate::memory_size::MemoryLayout;



pub struct AssemblyMetadata {
    pub(crate) asm: String,
    pub(crate) resultant_stack: MemoryLayout
}