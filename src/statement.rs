use crate::{compound_statement::ScopeStatements, control_flow_statement::ControlFlowChange, lexer::{token_savepoint::TokenQueueSlice, token_walk::TokenQueue}};
use std::fmt::Write;

#[derive(Debug)]
pub enum Statement {
    //LABEL,//for goto, or switch cases
    //EXPRESSION(Expression),
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
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice) -> Option<(Statement, TokenQueueSlice)> {
        let curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        if let Some((ss, remaining_tokens)) = ScopeStatements::try_consume(tokens_queue, &curr_queue_idx){
            return Some((Self::COMPOUND(ss), remaining_tokens));
        }

        if let Some((command, remaining_tokens)) = ControlFlowChange::try_consume(tokens_queue, &curr_queue_idx){
            return Some((Self::CONTROLFLOW(command), remaining_tokens));
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
        }

        return result;
    }
}