use crate::{lexer::{token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, number_literal::NumberLiteral, operator::Operator};
use std::fmt::Write;

#[derive(Debug)]
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
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice) -> Option<(Expression, TokenQueueSlice)> {
        PartialExpr::try_consume(tokens_queue, previous_queue_idx)//call helper enum to do the work for us
    }

    /**
     * puts the result of the expression in rax
     */
    pub fn generate_assembly(&self) -> String{
        let mut result = String::new();

        match self {
            Expression::NUMBER(number_literal) => {
                writeln!(result, "mov rax, {}", number_literal.nasm_format()).unwrap();
            },
        }

        result
    }
}

/**
 * used for parsing expressions, where some parts are yet to be parsed
 */
enum PartialExpr {
    EXPRESSION(Expression),
    UNPROCESSED(TokenQueueSlice)//specifies a slice of unprocessed tokens
}

impl PartialExpr {
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice) -> Option<(Expression, TokenQueueSlice)>  {
        let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        match curr_queue_idx.get_slice_size() {
            0 => panic!("not expecting this, maybe it is not an expression"),

            1 => {
                //1 token left, check if it is a number
                if let Token::NUMBER(num) = tokens_queue.peek(& curr_queue_idx)? {
                    tokens_queue.consume(&mut curr_queue_idx);
                    return Some((Expression::NUMBER(num), curr_queue_idx));
                }
                None
            },

            _ => {
                //TODO handle brackets outside of operator

                //find highest precendence level
                let max_precedence = tokens_queue.get_slice(&curr_queue_idx).iter()
                    .filter_map(|x| {
                        if let Token::OPERATOR(op) = x {Some(op.get_precedence_level())} else {None} //get the precedence level if it is an operator, else skip
                    })
                    .fold(std::i32::MIN, |a,b| a.max(b));

                //find which direction 
                let precedence_direction = Operator::get_precedence_direction(max_precedence);


                todo!("look for '+' and the like")
            }
        }
    }
}