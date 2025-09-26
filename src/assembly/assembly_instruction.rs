use colored::Colorize;
use stack_management::baked_stack_frame::BakedSimpleStackFrame;

use crate::debugging::IRDisplay;

use super::operation::IROperation;


/**
 * this is a line of assembly complete with comments and operation
 */
#[derive(Clone)]
pub struct IRInstruction {
    comment: Option<String>,
    operation: IROperation
}

impl IRInstruction {
    pub fn generate(operation: IROperation) -> Self {
        IRInstruction { comment: None, operation }
    }
    pub fn generate_with_comment(operation: IROperation, comment: String) -> Self {
        IRInstruction { comment: Some(comment), operation }
    }

    pub fn emit_assembly(&self, stack: &BakedSimpleStackFrame) -> String{
        if let Some(comment) = &self.comment {
            format!("{} ; {}", self.operation.to_text(stack), comment)
        } else {
            self.operation.to_text(stack).to_string()
        }
    }
}

impl IRDisplay for IRInstruction {
    fn display_ir(&self) -> String {
        if let Some(comment) = &self.comment {
            let comment = format!("//{}", comment).bright_green();
            format!("{} {}", self.operation.display_ir(), comment)
        } else {
            self.operation.display_ir()
        }
    }
}