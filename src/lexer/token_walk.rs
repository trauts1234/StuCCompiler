use super::{token::Token, token_savepoint::TokenQueueSlice, punctuator::Punctuator};

/**
 * this steps through each token
 */
pub struct TokenQueue {
    pub(crate) tokens: Vec<Token>,
}

/**
 * a struct for where to search when searching for a closure in a TokenQueue
 */
pub struct TokenSearchType {
    pub(crate) skip_in_curly_brackets: bool,
    pub(crate) skip_in_square_brackets: bool,
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

    /**
     * peeks the token at the end of the token queue slice
     */
    pub fn peek_back(&self, location: &TokenQueueSlice) -> Option<Token> {
        let max_idx = location.max_index-1;

        if location.index >= location.max_index || location.max_index > self.tokens.len() {
            return None;
        }

        Some(self.tokens[max_idx].clone())
    }

    pub fn consume(&self, location: &mut TokenQueueSlice) -> Option<Token> {
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
     * returns a list of zero size slices, that have an index of each token matching the predicate
     */
    pub fn find_closure_matches<Matcher>(&self, slice: &TokenQueueSlice, scan_backwards: bool, predicate: Matcher, exclusions: &TokenSearchType) -> Vec<TokenQueueSlice> 
    where Matcher: Fn(&Token) -> bool
    {
        let mut bracket_depth = 0;//how many sets of brackets I am in

        let min_index = 0.max(slice.index);//either start of list, or start of slice
        let max_index = self.tokens.len().min(slice.max_index);//end of array or end of slice

        //conditionally reverse the loop
        let range: Box<dyn Iterator<Item = _>> = if scan_backwards {Box::new((min_index..max_index).rev())} else {Box::new(min_index..max_index)};
        let mut found_matches = Vec::new();

        for i in range {
            match &self.tokens[i] {
                Token::PUNCTUATOR(Punctuator::OPENCURLY) if exclusions.skip_in_curly_brackets => {
                    bracket_depth += 1;//I should avoid being in brackets if that flag is set
                }
                Token::PUNCTUATOR(Punctuator::CLOSECURLY) if exclusions.skip_in_curly_brackets => {
                    bracket_depth -= 1;
                }

                Token::PUNCTUATOR(Punctuator::OPENSQUARE) if exclusions.skip_in_square_brackets => {
                    bracket_depth += 1;
                }
                Token::PUNCTUATOR(Punctuator::CLOSESQUARE) if exclusions.skip_in_square_brackets => {
                    bracket_depth -= 1;
                }

                tok if bracket_depth == 0 && predicate(&tok) => {//outside of brackets, matching the predicate, and bracket depth was not just changed
                    found_matches.push(TokenQueueSlice {index: i, max_index: i+1});
                }
                
                _ => {}
            }
        }

        found_matches
    }

    /**
     * splits the section of the token queue within bounds by the index specified
     * does not include the split index in either slice
     */
    pub fn split_to_slices(&self, split_location: usize, bounds: &TokenQueueSlice) -> (TokenQueueSlice, TokenQueueSlice) {
        (
            TokenQueueSlice{
                index: bounds.index, max_index: split_location
            },//up to but not including index
            TokenQueueSlice{
                index: split_location + 1, max_index: bounds.max_index
            }//from past index to end of bounds
        )
    }

    pub fn split_outside_parentheses<Matcher>(&self, bounds: &TokenQueueSlice, predicate: Matcher) -> Vec<TokenQueueSlice> 
    where Matcher: Fn(&Token) -> bool
    {
        let mut inside_parentheses = 0;
        let mut prev_slice_end = bounds.index;//start slice at beginning of the bounds
        let mut slices = Vec::new();

        for i in bounds.index..bounds.max_index {
            match &self.tokens[i] {
                Token::PUNCTUATOR(x) if *x == Punctuator::OPENCURLY => {
                    inside_parentheses += 1;
                },
                Token::PUNCTUATOR(x) if *x == Punctuator::CLOSECURLY => {
                    inside_parentheses -= 1;
                },
                
                tok if predicate(tok) && inside_parentheses == 0 => {//match predicate and not inside parentheses
                    slices.push(TokenQueueSlice{index: prev_slice_end, max_index: i});
                    prev_slice_end = i + 1;
                },

                _ => {}
            }
        }

        slices.push(TokenQueueSlice{index: prev_slice_end, max_index: bounds.max_index});//add remaining characters

        slices
    }

    /**
     * given a slice that starts with an open curly, and contains a close curly
     * consume from location all the parenthesis data including ( )
     * and return a slice of all the tokens inside the parenthesis excluding ( )
     */
    pub fn consume_inside_parenthesis(&self, location: &mut TokenQueueSlice) -> TokenQueueSlice {
        assert!(self.peek(location) == Some(Token::PUNCTUATOR(Punctuator::OPENCURLY)));

        let mut inside_parentheses = 0;
        let parenthesis_open_idx = location.index;

        for i in parenthesis_open_idx..location.max_index {
            match &self.tokens[i] {
                Token::PUNCTUATOR(Punctuator::OPENCURLY) => {
                    inside_parentheses += 1;
                },

                Token::PUNCTUATOR(Punctuator::CLOSECURLY) => {
                    inside_parentheses -= 1;

                    if inside_parentheses == 0 {
                        //bracket takes us outside of all brackets, must be matching bracket
                        location.index = i+1;//consume until the token after close bracket
                        return TokenQueueSlice {index: parenthesis_open_idx+1, max_index: i};//return slice of inside the brackets
                    }
                },

                _ => {}
            }
        }

        panic!("close bracket not found")
    }

    /**
     * detects if the slice passed is all the text: (anything) including brackets on both sides
     */
    pub fn slice_is_parenthesis(&self, slice: &TokenQueueSlice) -> bool {
        //ensure the start is an open bracket
        if self.peek(slice) != Some(Token::PUNCTUATOR(Punctuator::OPENCURLY)){
            return false;
        }

        //sum open and close brackets
        let resultant_bracket_level = 
        self.tokens[slice.index..slice.max_index].iter()
            .map(|x| match x {
                Token::PUNCTUATOR(Punctuator::OPENCURLY) => 1,
                Token::PUNCTUATOR(Punctuator::CLOSECURLY) => -1,
                _ => 0
            }).sum::<i32>();

        //ensure this equals 0
        return resultant_bracket_level == 0;
    }

    pub fn find_matching_open_bracket(&self, close_idx: usize) -> usize {
        let mut bracket_depth = 0;

        let (open_bracket, close_bracket) = match self.tokens[close_idx] {
            Token::PUNCTUATOR(Punctuator::CLOSESQUARE) => (Punctuator::OPENSQUARE, Punctuator::CLOSESQUARE),
            Token::PUNCTUATOR(Punctuator::CLOSECURLY) => (Punctuator::OPENCURLY, Punctuator::CLOSECURLY),
            _ => {panic!("unknown close bracket that I am trying to match")}
        };

        for i in (0..=close_idx).rev() {
            match &self.tokens[i] {
                Token::PUNCTUATOR(br) if *br == close_bracket => {bracket_depth += 1;},//add here as I am scanning backwards
                Token::PUNCTUATOR(br) if *br == open_bracket => {bracket_depth -= 1;},
                _ => {}
            }

            if bracket_depth == 0 {return i;}
        }

        panic!("matching [/( not found");
    }

    pub fn find_matching_close_bracket(&self, open_idx: usize) -> usize {
        let mut bracket_depth = 0;

        let (open_bracket, close_bracket) = match self.tokens[open_idx] {
            Token::PUNCTUATOR(Punctuator::OPENSQUARE) => (Punctuator::OPENSQUARE, Punctuator::CLOSESQUARE),
            Token::PUNCTUATOR(Punctuator::OPENCURLY) => (Punctuator::OPENCURLY, Punctuator::CLOSECURLY),
            Token::PUNCTUATOR(Punctuator::OPENSQUIGGLY) => (Punctuator::OPENSQUIGGLY, Punctuator::CLOSESQUIGGLY),
            _ => {panic!("unknown open bracket that I am trying to match")}
        };

        for i in open_idx..self.tokens.len() {
            match &self.tokens[i] {
                Token::PUNCTUATOR(br) if *br == open_bracket => {bracket_depth += 1;},
                Token::PUNCTUATOR(br) if *br == close_bracket => {bracket_depth -= 1;},
                _ => {}
            }

            if bracket_depth == 0 {assert!(self.tokens[i] == Token::PUNCTUATOR(close_bracket));return i;}
        }

        println!("{:?}", &self.tokens[open_idx..]);
        panic!("matching )/] not found");
    }
}