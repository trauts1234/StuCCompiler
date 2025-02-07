use crate::{ast_metadata::ASTMetadata, compound_statement::ScopeStatements, control_flow_statement::ControlFlowChange, expression::Expression, lexer::{token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, stack_variables::StackVariables};
use std::fmt::Write;

#[derive(Debug)]
pub enum Statement {
    //LABEL,//for goto, or switch cases
    EXPRESSION(Expression),//TODO
    COMPOUND(ScopeStatements),//this is a scope (not nescessarily for a function)
    //SELECTION,
    //ITERATION,
    CONTROLFLOW(ControlFlowChange),
}

impl Statement {
    /**
     * tries to parse the tokens queue starting at previous_queue_idx, to find a statement
     * returns a statement and the remaining tokens as a queue location, else none
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, local_variables: &StackVariables) -> Option<ASTMetadata<Statement>> {
        let curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        if let Some(ASTMetadata{resultant_tree, remaining_slice, extra_stack_used}) = ScopeStatements::try_consume(tokens_queue, &curr_queue_idx, local_variables){
            return Some(ASTMetadata{resultant_tree: Self::COMPOUND(resultant_tree), remaining_slice, extra_stack_used});
        }

        if let Some(ASTMetadata{resultant_tree, remaining_slice, extra_stack_used}) = ControlFlowChange::try_consume(tokens_queue, &curr_queue_idx, local_variables){
            return Some(ASTMetadata{resultant_tree: Self::CONTROLFLOW(resultant_tree), remaining_slice, extra_stack_used});
        }

        

        None
    }

    pub fn generate_assembly(&self) -> String {
        let mut result = String::new();

        match self {
            Self::COMPOUND(scope) => {
                write!(result, "{}", scope.generate_assembly()).unwrap();
            }
            Self::CONTROLFLOW(command) => {
                write!(result, "{}", command.generate_assembly()).unwrap();
            }
            Self::EXPRESSION(expr) => {
                write!(result, "{}", expr.generate_assembly()).unwrap();
            }
        }

        return result;
    }
}