use super::{token::Token, token_savepoint::TokenQueueLocation};

/**
 * this steps through each token
 */
pub struct TokenQueue {
    tokens: Vec<Token>,
}

impl TokenQueue {
    pub fn new(token_list: Vec<Token>) -> TokenQueue{
        TokenQueue {
            tokens:token_list,
        }
    }

    /**
     * returns the next token that needs to be consumed
     */
    pub fn peek(&self, location: &TokenQueueLocation) -> Option<Token> {
        let next_idx = location.get_index();

        if next_idx >= self.tokens.len(){
            return None;//run out of tokens
        }

        Some(self.tokens[next_idx].clone())
    }

    pub fn consume(&mut self, location: &mut TokenQueueLocation) -> Option<Token> {
        let next = self.peek(&location);
        location.next();
        if location.get_index() > self.tokens.len(){
            panic!("continued consuming tokens after end of array")
        }
        return next;
    }
}