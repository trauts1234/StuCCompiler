use super::{token::Token, token_savepoint::TokenQueueSlice};

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
    pub fn peek(&self, location: &TokenQueueSlice) -> Option<Token> {
        let next_idx = location.get_index();

        if next_idx >= self.tokens.len() || next_idx >= location.get_slice_max_idx(){
            return None;//run out of tokens
        }

        Some(self.tokens[next_idx].clone())
    }

    pub fn consume(&mut self, location: &mut TokenQueueSlice) -> Option<Token> {
        let next = self.peek(&location);
        location.next();
        if location.get_index() > self.tokens.len() || location.get_index() > location.get_slice_max_idx(){
            panic!("continued consuming tokens after end of array or past end of allowed slice")
        }
        return next;
    }

    pub fn get_slice(&self, slice: &TokenQueueSlice) -> &[Token] {
        &self.tokens[slice.get_index()..slice.get_slice_max_idx()]
    }
}