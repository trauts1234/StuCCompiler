use crate::{asm_boilerplate, expression::Expression, lexer::{token::Token, token_savepoint::TokenQueueLocation, token_walk::TokenQueue}};
use std::fmt::Write;

/**
 * this handles break, continue and return statements
 */
#[derive(Debug)]
pub enum ControlFlowChange {
    RETURN(Option<Expression>)
}

impl ControlFlowChange {
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueLocation) -> Option<(ControlFlowChange, TokenQueueLocation)> {
        let mut curr_queue_idx = TokenQueueLocation::from_previous_savestate(previous_queue_idx);

        let kw = if let Some(Token::KEYWORD(x)) = tokens_queue.consume(&mut curr_queue_idx) {x} else {return None;};
        
        match kw.as_str() {
            "return" => {

                //try and match with an expression for what to return, but don't worry if not as some functions return void
                let ret_value = match Expression::try_consume(tokens_queue, &curr_queue_idx) {
                    None => None,
                    Some((return_expr, remaining_tokens)) => {
                        curr_queue_idx = remaining_tokens;
                        Some(return_expr)
                    }
                };

                //ensure return ends with ;
                if Token::PUNCTUATION(";".to_owned()) != tokens_queue.consume(&mut curr_queue_idx)?{
                    return None;
                }

                Some((Self::RETURN(ret_value), curr_queue_idx))
            }
            _ => None
        }
    }

    pub fn generate_assembly(&self) -> String {
        let mut result = String::new();

        match self {
            ControlFlowChange::RETURN(expression) => {
                if let Some(expr) = expression {
                    write!(result, "{}", expr.generate_assembly()).unwrap();
                }
                //warning: ensure result is in the correct register and correctly sized
                write!(result, "{}", asm_boilerplate::func_exit_boilerplate()).unwrap();
            },
        }

        result
    }
}