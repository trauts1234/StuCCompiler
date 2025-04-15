use memory_size::MemorySize;

//just a string member, in NASM-friendly format already
#[derive(Clone, Debug)]
pub struct ImmediateValue(pub String);

impl ImmediateValue {
    pub fn generate_name(&self) -> String {
        self.0.clone()
    }
}

//extend functionality of memory layout to add extra useful function
pub trait MemorySizeExt {
    /**
     * converts this number as a number of bytes into an immediate value
     */
    fn as_imm(&self) -> ImmediateValue;
}

impl MemorySizeExt for MemorySize {
    fn as_imm(&self) -> ImmediateValue {
        ImmediateValue(self.size_bytes().to_string())
    }
}