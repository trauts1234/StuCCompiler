use super::operation::AsmOperation;


/**
 * this is a line of assembly complete with comments and operation
 */
#[derive(Clone)]
pub struct AsmInstruction {
    comment: Option<String>,
    operation: AsmOperation
}

impl AsmInstruction {
    pub fn generate(operation: AsmOperation) -> Self {
        AsmInstruction { comment: None, operation }
    }
    pub fn generate_with_comment(operation: AsmOperation, comment: String) -> Self {
        AsmInstruction { comment: Some(comment), operation }
    }

    pub fn emit_assembly(&self) -> String{
        if let Some(comment) = &self.comment {
            format!("{} ; {}", self.operation.to_text(), comment)
        } else {
            self.operation.to_text()
        }
    }
}