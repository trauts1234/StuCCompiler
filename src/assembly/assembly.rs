use super::{assembly_instruction::AsmInstruction, operation::AsmOperation};



pub struct Assembly {
    lines: Vec<AsmInstruction>
}

impl Assembly {
    /**
     * makes an empty assembly file
     */
    pub fn make_empty() -> Self {
        Assembly { lines: Vec::new() }
    }

    pub fn merge(&mut self, other: &Assembly) {
        self.lines.extend(other.lines.iter().cloned());
    }
    
    pub fn add_comment<S: AsRef<str>>(&mut self, comment: S) {
        self.lines.push(AsmInstruction::generate_with_comment(AsmOperation::BLANK, comment.as_ref().to_string()));
    }
    pub fn add_commented_instruction<S: AsRef<str>>(&mut self, operation: AsmOperation, comment: S) {
        self.lines.push(AsmInstruction::generate_with_comment(operation, comment.as_ref().to_string()));
    }
    pub fn add_instruction(&mut self, operation: AsmOperation) {
        self.lines.push(AsmInstruction::generate(operation));
    }

    pub fn get_lines(&self) -> &[AsmInstruction] {
        &self.lines
    }
}