use crate::{lexer::{token::{self, Token}, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, number_literal::NumberLiteral, operator::Operator};
use std::fmt::Write;

#[derive(Debug)]
pub enum Expression {
    //RVALUE(RValue),
    NUMBER(NumberLiteral),
    BINARYEXPR(Box<Expression>, Operator, Box<Expression>)
    //ASSIGNMENT(LValue, Operator, Box<Expression>)// a = b;
}

impl Expression {
    /**
     * tries to parse the tokens queue starting at previous_queue_idx, to find an expression
     * returns an expression(entirely consumed), else none
     */
    pub fn try_consume_whole_expr(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice) -> Option<Expression> {
        let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        match curr_queue_idx.get_slice_size() {
            0 => panic!("not expecting this, maybe it is not an expression"),

            1 => {
                //1 token left, check if it is a number
                if let Token::NUMBER(num) = tokens_queue.peek(& curr_queue_idx)? {
                    tokens_queue.consume(&mut curr_queue_idx);
                    return Some(Expression::NUMBER(num));
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

                //find which direction the operators should be considered
                let associative_direction = Operator::get_associativity_direction(max_precedence);

                //make a closure that detects tokens that match what we want
                let operator_matching_closure = |x: &Token| {
                    match x {
                        Token::OPERATOR(op) => {op.get_precedence_level() == max_precedence},
                        _ => false
                    }
                };

                //find first occurence of this operator, taking into account which way we have to search the array
                let first_operator_location = tokens_queue.find_closure_in_slice(&curr_queue_idx, !associative_direction, operator_matching_closure).unwrap();

                //split to before and after the operator
                let (left_part, right_part) = tokens_queue.split_to_slices(&first_operator_location, &curr_queue_idx);

                //try and parse the left and right hand sides, propogating errors
                let parsed_left = Expression::try_consume_whole_expr(tokens_queue, &left_part)?;
                let parsed_right = Expression::try_consume_whole_expr(tokens_queue, &right_part)?;

                let operator = match tokens_queue.peek(&first_operator_location).unwrap() {
                    Token::OPERATOR(op) => op,
                    _ => panic!("operator token is not an operator")
                };

                Some(Expression::BINARYEXPR(Box::new(parsed_left), operator, Box::new(parsed_right)))
            }
        }
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
            Expression::BINARYEXPR(lhs, operator, rhs) => {
                todo!()
            },
        }

        result
    }
}