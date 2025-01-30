use crate::{expression::Expression, lexer::{token::Token, token_savepoint::TokenQueueLocation, token_walk::TokenQueue}};


/**
 * this handles break, continue and return statements
 */
pub enum ControlFlowChange {
    RETURN(Option<Expression>)
}

impl ControlFlowChange {
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueLocation) -> Option<(ControlFlowChange, TokenQueueLocation)> {
        let mut curr_queue_idx = TokenQueueLocation::from_previous_savestate(previous_queue_idx);

        let kw = if let Some(Token::KEYWORD(x)) = tokens_queue.consume(&mut curr_queue_idx) {x} else {return None;};
        
        match kw.as_str() {
            "return" => {
                todo!("parse return data")
            }
            _ => todo!()
        }
    }
}