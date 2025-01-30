use std::collections::VecDeque;

use crate::{block_statement::StatementOrDeclaration, lexer::{token::{self, Token}, token_walk::TokenQueue}};


/**
 * this represents all the code inside a scope (i.e function definition)
 */
pub struct ScopeStatements {
    statements: Vec<StatementOrDeclaration>
}

impl ScopeStatements {
    pub fn try_consume_statements(tokens_queue: &mut TokenQueue) -> Option<ScopeStatements> {
        tokens_queue.save_checkpoint();

        let mut statements = Vec::new();

        if Some(Token::PUNCTUATION("{".to_owned())) != tokens_queue.peek() {
            tokens_queue.pop_checkpoint();
            return None;//not enclosed in { }, so can't be a scope
        }
        tokens_queue.consume();

        //greedily consume as many statements as possible
        while let Some(statement_or_decl) = StatementOrDeclaration::try_consume(tokens_queue) {
            statements.push(statement_or_decl);
        }

        tokens_queue.pop_checkpoint();
        if statements.len() == 0 {
            None
        } else {
            Some(ScopeStatements {
                statements
            })
        }
    }
}