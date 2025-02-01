use crate::{lexer::{token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, statement::Statement};


/**
 * This represents either a statement or variable creation.
 * The sort of things found in functions
 */
#[derive(Debug)]
pub enum StatementOrDeclaration {
    STATEMENT(Statement),
    //DECLARATION(Declaration)
}

impl StatementOrDeclaration {
    /**
     * tries to parse the tokens queue starting at previous_queue_idx, to find either a declaration or statement
     * returns a StatementOrDeclaration and the remaining tokens as a queue location, else none
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice) -> Option<(StatementOrDeclaration, TokenQueueSlice)> {
        let curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        if let Some((stat, remaining_tokens)) = Statement::try_consume(tokens_queue, &curr_queue_idx) {
            return Some((Self::STATEMENT(stat), remaining_tokens));
        }

        None
    }

    pub fn generate_assembly(&self) -> String {
        match self {
            Self::STATEMENT(statement) => statement.generate_assembly()
        }
    }
}