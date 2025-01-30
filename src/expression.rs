use crate::{lexer::{token_savepoint::TokenQueueLocation, token_walk::TokenQueue}, token::{number_literal::NumberLiteral, operator::Operator}};


pub enum Expression {
    RVALUE(RValue),
    NUMBER(NumberLiteral),
    ASSIGNMENT(LValue, Operator, Box<Expression>)// a = b;
}

impl Expression {
    /**
     * tries to parse the tokens queue starting at previous_queue_idx, to find an expression
     * returns an expression and the remaining tokens as a queue location, else none
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueLocation) -> Option<(Expression, TokenQueueLocation)> {
        todo!()
        //try and find if it is a number
    }
}