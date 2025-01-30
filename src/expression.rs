use crate::{lexer::{token::Token, token_savepoint::TokenQueueLocation, token_walk::TokenQueue}, token::number_literal::NumberLiteral};


pub enum Expression {
    //RVALUE(RValue),
    NUMBER(NumberLiteral),
    //ASSIGNMENT(LValue, Operator, Box<Expression>)// a = b;
}

impl Expression {
    /**
     * tries to parse the tokens queue starting at previous_queue_idx, to find an expression
     * returns an expression and the remaining tokens as a queue location, else none
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueLocation) -> Option<(Expression, TokenQueueLocation)> {
        let mut curr_queue_idx = TokenQueueLocation::from_previous_savestate(previous_queue_idx);

        if let Token::NUMBER(num) = tokens_queue.peek(&curr_queue_idx)? {
            tokens_queue.consume(&mut curr_queue_idx);
            return Some((
                Expression::NUMBER(NumberLiteral::try_new(&num).unwrap()), curr_queue_idx));
        }

        None
    }
}