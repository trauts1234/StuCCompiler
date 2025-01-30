use super::token::Token;

/**
 * this steps through each token
 */
pub struct TokenQueue {
    tokens: Vec<Token>,
    next_token_savepoints: Vec<usize>
}

impl TokenQueue {
    pub fn new(token_list: Vec<Token>) -> TokenQueue{
        TokenQueue {
            tokens:token_list,
            next_token_savepoints: vec![0]
        }
    }

    /**
     * returns the next token that needs to be consumed
     */
    pub fn peek(&self) -> Option<Token> {
        let next_idx = *self.next_token_savepoints.last().unwrap();

        if next_idx >= self.tokens.len(){
            return None;//run out of tokens
        }

        Some(self.tokens[next_idx].clone())
    }

    pub fn consume(&mut self) -> Option<Token> {
        let next = self.peek();
        *self.next_token_savepoints.last_mut().unwrap() += 1;
        return next;
    }

    /**
     * this saves how far the tokens have been consumed, so that it is possible to un-consume tokens later
     */
    pub fn save_checkpoint(&mut self) {
        let curr_idx = *self.next_token_savepoints.last().unwrap();
        self.next_token_savepoints.push(curr_idx);
    }

    /**
     * restore consume progress to back when the last save_checkpoint was called
     */
    pub fn pop_checkpoint(&mut self) {
        assert!(self.next_token_savepoints.len() > 1);
        self.next_token_savepoints.pop();
    }
}