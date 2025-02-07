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
        let next_idx = location.index;

        if next_idx >= self.tokens.len() || next_idx >= location.max_index{
            return None;//run out of tokens
        }

        Some(self.tokens[next_idx].clone())
    }

    pub fn consume(&mut self, location: &mut TokenQueueSlice) -> Option<Token> {
        let next = self.peek(&location);
        location.next();
        if location.index > self.tokens.len() || location.index > location.max_index{
            panic!("continued consuming tokens after end of array or past end of allowed slice")
        }
        return next;
    }

    pub fn get_slice(&self, slice: &TokenQueueSlice) -> &[Token] {
        let max_idx = slice.max_index.min(self.tokens.len());//whichever is smaller: list size, slice max index
        &self.tokens[slice.index..max_idx]
    }

    /**
     * tries to find where a closure is first true within the slice
     * returns a slice from the matched token to the end of the input slice
     */
    pub fn find_closure_in_slice<Matcher>(&self, slice: &TokenQueueSlice, scan_backwards: bool, predicate: Matcher) -> Option<TokenQueueSlice> 
    where Matcher: Fn(&Token) -> bool
    {
        let min_index = 0.max(slice.index);//either start of list, or start of slice
        let max_index = self.tokens.len().min(slice.max_index);//end of array or end of slice

        //conditionally reverse the loop
        let range: Box<dyn Iterator<Item = _>> = if scan_backwards {Box::new((min_index..max_index).rev())} else {Box::new(min_index..max_index)};

        for i in range {
            if predicate(&self.tokens[i]) {
                return Some(TokenQueueSlice{index: i, max_index: slice.max_index});
            }
        }

        None
    }

    /**
     * splits the section of the token queue within bounds by the index specified
     */
    pub fn split_to_slices(&self, split_location: &TokenQueueSlice, bounds: &TokenQueueSlice) -> (TokenQueueSlice, TokenQueueSlice) {
        (
            TokenQueueSlice{
                index: bounds.index, max_index: split_location.index
            },//up to but not including index
            TokenQueueSlice{
                index: split_location.index + 1, max_index: bounds.max_index
            }//from past index to end of bounds
        )
    }
}