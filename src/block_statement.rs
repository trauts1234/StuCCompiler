use crate::{lexer::token_walk::TokenQueue, statement::Statement};


/**
 * This represents either a statement or variable creation.
 * The sort of things found in functions
 */
pub enum StatementOrDeclaration {
    STATEMENT(Statement),
    //DECLARATION(Declaration)
}

impl StatementOrDeclaration {
    pub fn try_consume(tokens_queue: &mut TokenQueue) -> Option<StatementOrDeclaration> {
        tokens_queue.save_checkpoint();

        if let Some(stat) = Statement::try_consume_statement(tokens_queue) {
            tokens_queue.pop_checkpoint();
            return Some(Self::STATEMENT(stat));
        }

        tokens_queue.pop_checkpoint();
        None
    }
}