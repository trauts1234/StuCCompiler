
use crate::{block_statement::StatementOrDeclaration, lexer::{token::Token, token_savepoint::TokenQueueLocation, token_walk::TokenQueue}};


/**
 * this represents all the code inside a scope (i.e function definition)
 */
pub struct ScopeStatements {
    statements: Vec<StatementOrDeclaration>
}

impl ScopeStatements {
    /**
     * tries to parse the tokens queue starting at previous_queue_idx, to find a scope, for a function or other
     * returns a ScopeStatements and the remaining tokens as a queue location, else none
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueLocation) -> Option<(ScopeStatements, TokenQueueLocation)> {
        let mut curr_queue_idx = TokenQueueLocation::from_previous_savestate(previous_queue_idx);

        let mut statements = Vec::new();

        if Token::PUNCTUATION("{".to_owned()) != tokens_queue.consume(&mut curr_queue_idx)? {
            return None;//not enclosed in { }, so can't be a scope
        }

        //greedily consume as many statements as possible
        while let Some((statement_or_decl, remaining_tokens)) = StatementOrDeclaration::try_consume(tokens_queue, &curr_queue_idx) {
            statements.push(statement_or_decl);
            curr_queue_idx = remaining_tokens;//jump to next one
        }

        if statements.len() == 0 {
            return None;
        }

        if Token::PUNCTUATION("}".to_owned()) != tokens_queue.consume(&mut curr_queue_idx)? {
            return None;//not enclosed in { }, so can't be a scope
        }

        Some((ScopeStatements {
            statements
        }, curr_queue_idx))
    }
}