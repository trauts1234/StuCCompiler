use crate::{compilation_state::functions::FunctionList, expression::{self, Expression}, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, parse_data::ParseData};

#[derive(Clone)]
pub struct ArrayInitialisation {
    elements: Vec<Expression>,
}

impl ArrayInitialisation {
    /// parses initialisation like {1, 2, 3}
    pub fn try_consume_whole_expr(tokens_queue: &TokenQueue, previous_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, scope_data: &mut ParseData) -> Option<Self> {
        if !tokens_queue.slice_is_brackets(previous_queue_idx, Punctuator::OPENSQUIGGLY) {
            return None;//initialisation must be the whole slice
        }

        //strip the { }
        let curr_queue_idx = TokenQueueSlice {
            index: previous_queue_idx.index + 1,
            max_index: previous_queue_idx.max_index - 1,
        };

        let items = tokens_queue.split_outside_parentheses(&curr_queue_idx, |x| *x == Token::PUNCTUATOR(Punctuator::COMMA), &TokenSearchType::skip_all());

        let mut parsed = Vec::new();

        for slice in items {
            //try to convert each slice to an expression
            parsed.push(
                expression::try_consume_whole_expr(tokens_queue, &slice, accessible_funcs, scope_data)?//return None early if any slice is not an expression
            );
        }

        Some(
            ArrayInitialisation { elements: parsed }
        )
    }

    pub fn nth_item(&self, idx: usize) -> &Expression {
        //TODO return a 0 number literal or similar if index out of bounds: int x[100] = {0,1}; //for example
        &self.elements[idx]
    }
}