use crate::memory_size::MemoryLayout;


//just a string member, in NASM-friendly format already
#[derive(Clone, Debug)]
pub struct ImmediateValue(pub String);

impl ImmediateValue {
    pub fn generate_name(&self) -> String {
        self.0.clone()
    }
}

//extend functionality of memory layout to add extra useful function
impl MemoryLayout {
    /**
     * converts this number as a number of bytes into an immediate value
     */
    pub fn as_imm(&self) -> ImmediateValue {
        ImmediateValue(self.size_bytes().to_string())
    }
}