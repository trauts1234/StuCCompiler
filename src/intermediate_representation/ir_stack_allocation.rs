use memory_size::MemorySize;


#[derive(Debug, PartialEq, Eq)]
pub enum VarIdentifier {
    StackVariable {name: String},
    Temp {id: u32}
}

#[derive(Debug)]
pub struct VariableInfo {
    id: VarIdentifier,
    size: MemorySize
}

impl VariableInfo {
    pub fn identifier(&self) -> &VarIdentifier {
        &self.id
    }
}