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
        let max_idx = slice.get_slice_max_idx().min(self.tokens.len());//whichever is smaller: list size, slice max index
        &self.tokens[slice.get_index()..max_idx]
    }

    /**
     * tries to find where a closure is first true within the slice
     * returns a slice from the matched token to the end of the input slice
     */
    pub fn find_closure_in_slice<Matcher>(&self, slice: &TokenQueueSlice, scan_backwards: bool, predicate: Matcher) -> Option<TokenQueueSlice> 
    where Matcher: Fn(&Token) -> bool
    {
        let min_index = 0.max(slice.get_index());//either start of list, or start of slice
        let max_index = self.tokens.len().min(slice.get_slice_max_idx());//end of array or end of slice

        //conditionally reverse the loop
        let range: Box<dyn Iterator<Item = _>> = if scan_backwards {Box::new((min_index..max_index).rev())} else {Box::new(min_index..max_index)};

        for i in range {
            if predicate(&self.tokens[i]) {
                return Some(TokenQueueSlice::new_from_bounds(i, slice.get_slice_max_idx()));
            }
        }

        None
    }

    /**
     * splits the section of the token queue within bounds by the index specified
     */
    pub fn split_to_slices(&self, index: &TokenQueueSlice, bounds: &TokenQueueSlice) -> (TokenQueueSlice, TokenQueueSlice) {
        (
            TokenQueueSlice::new_from_bounds(bounds.get_index(), index.get_index()),//up to but not including index
            TokenQueueSlice::new_from_bounds(index.get_index() + 1, bounds.get_slice_max_idx())//from past index to end of bounds
        )
    }
}