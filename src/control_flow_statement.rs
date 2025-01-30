use crate::{expression::Expression, lexer::{token::{self, Token}, token_walk::TokenQueue}};


/**
 * this handles break, continue and return statements
 */
pub enum ControlFlowChange {
    RETURN(Option<Expression>)
}

impl ControlFlowChange {
    pub fn try_consume(tokens_queue: &mut TokenQueue) -> Option<ControlFlowChange> {
        tokens_queue.save_checkpoint();

        let kw = if let Some(Token::KEYWORD(x)) = tokens_queue.consume() {x} 
            else {tokens_queue.pop_checkpoint(); return None;};
        
        match kw {
            "return" => {
                if let
            }
        }
    }
}