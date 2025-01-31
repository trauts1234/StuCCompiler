use crate::{compound_statement::ScopeStatements, control_flow_statement::ControlFlowChange, expression::Expression, lexer::{token_savepoint::TokenQueueLocation, token_walk::TokenQueue}};

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
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueLocation) -> Option<(Statement, TokenQueueLocation)> {
        let curr_queue_idx = TokenQueueLocation::from_previous_savestate(previous_queue_idx);

        if let Some((ss, remaining_tokens)) = ScopeStatements::try_consume(tokens_queue, &curr_queue_idx){
            return Some((Self::COMPOUND(ss), remaining_tokens));
        }

        if let Some((command, remaining_tokens)) = ControlFlowChange::try_consume(tokens_queue, &curr_queue_idx){
            return Some((Self::CONTROLFLOW(command), remaining_tokens));
        }

        None
    }
}