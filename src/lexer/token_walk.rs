use crate::parse_data::ParseData;

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
    pub(crate) skip_in_squiggly_brackets: bool,
}

impl TokenSearchType {
    pub fn skip_nothing() -> TokenSearchType {
        TokenSearchType { skip_in_curly_brackets: false, skip_in_square_brackets: false, skip_in_squiggly_brackets: false }
    }
    pub fn skip_all() -> TokenSearchType {
        TokenSearchType { skip_in_curly_brackets: true, skip_in_square_brackets: true, skip_in_squiggly_brackets: true }
    }

    /// returns true if the bracket is the start of a portion that needs to be skipped according to the search type
    pub fn is_skippable_open_bracket(&self, punctuator: &Punctuator) -> bool {
        match punctuator {
            Punctuator::OPENCURLY if self.skip_in_curly_brackets => true,
            Punctuator::OPENSQUARE if self.skip_in_square_brackets => true,
            Punctuator::OPENSQUIGGLY if self.skip_in_squiggly_brackets => true,
            _ => false
        }
    }

    ///returns true if the bracket is a close bracket ending a portion that needed to be skipped
    pub fn is_skippable_close_bracket(&self, punctuator: &Punctuator) -> bool {
        match punctuator {
            Punctuator::CLOSECURLY if self.skip_in_curly_brackets => true,
            Punctuator::CLOSESQUARE if self.skip_in_square_brackets => true,
            Punctuator::CLOSESQUIGGLY if self.skip_in_squiggly_brackets => true,
            _ => false
        }
    }
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
    pub fn peek(&self, location: &TokenQueueSlice, scope_data: &ParseData) -> Option<Token> {
        let next_idx = location.index;

        if self.no_remaining_tokens(location){
            return None;//run out of tokens
        }

        Some(
            substitute_token(self.tokens[next_idx].clone(), scope_data)
        )
    }

    /**
     * peeks a token without substituting in enum variants
     */
    pub fn peek_raw(&self, location: &TokenQueueSlice) -> Option<Token> {
        let next_idx = location.index;

        if self.no_remaining_tokens(location){
            return None;//run out of tokens
        }

        Some(
            self.tokens[next_idx].clone()
        )
    }

    pub fn no_remaining_tokens(&self, location: &TokenQueueSlice) -> bool {
        let next_idx = location.index;

        //consumed all tokens         or the slice is empty
        next_idx >= self.tokens.len() || next_idx >= location.max_index
    }

    /**
     * peeks the token at the end of the token queue slice
     */
    pub fn peek_back(&self, location: &TokenQueueSlice, scope_data: &ParseData) -> Option<Token> {
        let max_idx = location.max_index-1;

        if location.index >= location.max_index || location.max_index > self.tokens.len() {
            return None;
        }

        Some(substitute_token(self.tokens[max_idx].clone(), scope_data))
    }

    pub fn consume(&self, location: &mut TokenQueueSlice, scope_data: &ParseData) -> Option<Token> {
        let next = self.peek(&location, scope_data);
        location.next();
        if location.index > self.tokens.len() || location.index > location.max_index{
            panic!("continued consuming tokens after end of array or past end of allowed slice")
        }
        return next;
    }

    pub fn display_slice(&self, slice: &TokenQueueSlice) -> String {
        let max_idx = slice.max_index.min(self.tokens.len());//whichever is smaller: list size, slice max index

        self.tokens[slice.index..max_idx]
        .iter()
        .fold(String::new(),
            |acc, x| acc + &format!("{}", x)
        )
    }

    pub fn is_slice_inbounds(&self, slice: &TokenQueueSlice) -> bool {
        assert!(slice.index <= slice.max_index);
        slice.index <= self.tokens.len() && slice.max_index <= self.tokens.len()
    }

    /**
     * tries to find where a closure is first true within the slice
     * returns a slice from the matched token to the end of the input slice
     */
    pub fn find_closure_matches<Matcher>(&self, slice: &TokenQueueSlice, scan_backwards: bool, predicate: Matcher, exclusions: &TokenSearchType) -> Option<TokenQueueSlice> 
    where Matcher: Fn(&Token) -> bool
    {
        let mut bracket_depth = 0;//how many sets of brackets I am in
        let min_index = 0.max(slice.index);//either start of list, or start of slice
        let max_index = self.tokens.len().min(slice.max_index);//end of array or end of slice

        //conditionally reverse the loop
        let range: Box<dyn Iterator<Item = _>> = if scan_backwards {Box::new((min_index..max_index).rev())} else {Box::new(min_index..max_index)};

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

                Token::PUNCTUATOR(Punctuator::OPENSQUIGGLY) if exclusions.skip_in_squiggly_brackets => {
                    bracket_depth += 1;
                }
                Token::PUNCTUATOR(Punctuator::CLOSESQUIGGLY) if exclusions.skip_in_squiggly_brackets => {
                    bracket_depth -= 1;
                }

                tok if bracket_depth == 0 && predicate(&tok) => {//outside of brackets, matching the predicate, and bracket depth was not just changed
                    return Some(TokenQueueSlice{index: i, max_index: slice.max_index});
                }
                
                _ => {}
            }
        }

        None
    }

    /**
     * returns a list of zero size slices, that have an index of each token matching the predicate
     */
    pub fn split_by_closure_matches<Matcher>(&self, slice: &TokenQueueSlice, scan_backwards: bool, predicate: Matcher, exclusions: &TokenSearchType) -> Vec<usize> 
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
                Token::PUNCTUATOR(x) if exclusions.is_skippable_open_bracket(x) => bracket_depth += 1,
                Token::PUNCTUATOR(x) if exclusions.is_skippable_close_bracket(x) => bracket_depth -= 1,

                tok if bracket_depth == 0 && predicate(&tok) => {//outside of brackets, matching the predicate, and bracket depth was not just changed
                    found_matches.push(i);
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

    /// - returns a list of slices, representing all slices split by predicate, with the item matching the predicate removed, just like string.split()
    /// - if no matches are found, ```vec![bounds.clone()]``` is returned
    pub fn split_outside_parentheses<Matcher>(&self, bounds: &TokenQueueSlice, predicate: Matcher, exclusions: &TokenSearchType) -> Vec<TokenQueueSlice> 
    where Matcher: Fn(&Token) -> bool
    {
        let mut inside_parentheses = 0;
        let mut prev_slice_end = bounds.index;//start slice at beginning of the bounds
        let mut slices = Vec::new();

        for i in bounds.index..bounds.max_index {
            match &self.tokens[i] {
                Token::PUNCTUATOR(x) if exclusions.is_skippable_open_bracket(x) => inside_parentheses += 1,
                Token::PUNCTUATOR(x) if exclusions.is_skippable_close_bracket(x) => inside_parentheses -= 1,
                
                tok if predicate(tok) && inside_parentheses == 0 => {//match predicate and not inside parentheses
                    slices.push(TokenQueueSlice{index: prev_slice_end, max_index: i});
                    prev_slice_end = i + 1;
                },

                _ => {}
            }
            assert!(inside_parentheses >= 0);
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
        assert!(self.peek_raw(location) == Some(Token::PUNCTUATOR(Punctuator::OPENCURLY)));

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
    pub fn slice_is_brackets(&self, slice: &TokenQueueSlice, expected_open_bracket: Punctuator) -> bool {
        //ensure the start is an open bracket
        if self.peek_raw(slice) != Some(Token::PUNCTUATOR(expected_open_bracket)){
            return false;
        }

        let matching_close = self.find_matching_close_bracket(slice.index);

        return matching_close == slice.max_index-1;
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

        let (open_bracket, close_bracket) = match &self.tokens[open_idx] {
            Token::PUNCTUATOR(Punctuator::OPENSQUARE) => (Punctuator::OPENSQUARE, Punctuator::CLOSESQUARE),
            Token::PUNCTUATOR(Punctuator::OPENCURLY) => (Punctuator::OPENCURLY, Punctuator::CLOSECURLY),
            Token::PUNCTUATOR(Punctuator::OPENSQUIGGLY) => (Punctuator::OPENSQUIGGLY, Punctuator::CLOSESQUIGGLY),
            x => {panic!("unknown open bracket that I am trying to match: {:?}", x)}
        };

        for i in open_idx..self.tokens.len() {
            match &self.tokens[i] {
                Token::PUNCTUATOR(br) if *br == open_bracket => {bracket_depth += 1;},
                Token::PUNCTUATOR(br) if *br == close_bracket => {bracket_depth -= 1;},
                _ => {}
            }

            if bracket_depth == 0 {assert!(self.tokens[i] == Token::PUNCTUATOR(close_bracket));return i;}
        }

        panic!("{:?}\nmatching )/] not found", &self.tokens[open_idx..]);
    }
}

fn substitute_token(original: Token, scope_data: &ParseData) -> Token {
    match &original {
        Token::IDENTIFIER(x) => {
            if let Some(enum_value) = scope_data.enums.try_get_variant(&x) {
                Token::NUMBER(enum_value.clone())
            } else {
                original
            }
        }
        _ => original
    }
}