use crate::{lexer::{token_savepoint::TokenQueueLocation, token_walk::TokenQueue}, statement::Statement};


/**
 * This represents either a statement or variable creation.
 * The sort of things found in functions
 */
pub enum StatementOrDeclaration {
    STATEMENT(Statement),
    //DECLARATION(Declaration)
}

impl StatementOrDeclaration {
    /**
     * tries to parse the tokens queue starting at previous_queue_idx, to find either a declaration or statement
     * returns a StatementOrDeclaration and the remaining tokens as a queue location, else none
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueLocation) -> Option<(StatementOrDeclaration, TokenQueueLocation)> {
        let curr_queue_idx = TokenQueueLocation::from_previous_savestate(previous_queue_idx);

        if let Some((stat, remaining_tokens)) = Statement::try_consume(tokens_queue, &curr_queue_idx) {
            return Some((Self::STATEMENT(stat), remaining_tokens));
        }

        None
    }
}