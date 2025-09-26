use crate::debugging::IRDisplay;

use super::{assembly_instruction::IRInstruction, operation::IROperation};



pub struct IRCode {
    lines: Vec<IRInstruction>
}

impl IRCode {
    /**
     * makes an empty set of code
     */
    pub fn make_empty() -> Self {
        IRCode { lines: Vec::new() }
    }

    pub fn merge(&mut self, other: &IRCode) {
        self.lines.extend(other.lines.iter().cloned());
    }
    
    pub fn add_comment<S: AsRef<str>>(&mut self, comment: S) {
        self.lines.push(IRInstruction::generate_with_comment(IROperation::BLANK, comment.as_ref().to_string()));
    }
    pub fn add_commented_instruction<S: AsRef<str>>(&mut self, operation: IROperation, comment: S) {
        self.lines.push(IRInstruction::generate_with_comment(operation, comment.as_ref().to_string()));
    }
    pub fn add_instruction(&mut self, operation: IROperation) {
        self.lines.push(IRInstruction::generate(operation));
    }

    pub fn get_lines(&self) -> &[IRInstruction] {
        &self.lines
    }
}

impl IRDisplay for IRCode {
    fn display_ir(&self) -> String {
        self.lines.iter()
        .map(|x| x.display_ir())
        .collect:: <Vec<_>>()
        .join("\n")
    }
}