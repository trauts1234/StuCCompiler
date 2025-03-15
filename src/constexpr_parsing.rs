use crate::{lexer::{token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, number_literal::NumberLiteral, scope_data::ScopeData};

/**
 * folds a constant expression to a number
 */
pub fn consume_whole_constexpr(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ScopeData) -> Option<NumberLiteral> {
    if previous_queue_idx.get_slice_size() == 1 {
        if let Token::NUMBER(number) = tokens_queue.peek(previous_queue_idx, scope_data)? {
            return Some(number);
        }
        panic!("found 1 length slice that was not a number when consuming a constexpr");
    }
    panic!("could not parse constant expression");
}