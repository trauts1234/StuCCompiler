use memory_size::MemoryLayout;

use crate::{asm_boilerplate, ast_metadata::ASTMetadata, expression::Expression, lexer::{token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size, stack_variables::StackVariables};
use std::fmt::Write;

/**
 * this handles break, continue and return statements
 */
#[derive(Debug)]
pub enum ControlFlowChange {
    RETURN(Option<Expression>)
}

impl ControlFlowChange {
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, local_variables: &StackVariables) -> Option<ASTMetadata<ControlFlowChange>> {
        let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        let kw = if let Some(Token::KEYWORD(x)) = tokens_queue.consume(&mut curr_queue_idx) {x} else {return None;};
        
        match kw.as_str() {
            "return" => {

                //try to find semicolon at end of return statement
                let semicolon_idx = tokens_queue.find_closure_in_slice(&curr_queue_idx, false, |x| *x == Token::PUNCTUATION(";".to_owned()))?;

                let return_value_slice = TokenQueueSlice::new_from_bounds(curr_queue_idx.get_index(), semicolon_idx.get_index());//between return statement and ; non inclusive

                if return_value_slice.get_slice_size() == 0 {
                    //func returned nothing(void)
                    //return slice of all tokens after the semicolon
                    return Some(ASTMetadata{resultant_tree: Self::RETURN(None),remaining_slice: semicolon_idx.next_clone(), extra_stack_used: MemoryLayout::new()});
                }

                //try and match with an expression for what to return
                let ret_value = Expression::try_consume_whole_expr(tokens_queue, &return_value_slice, local_variables).unwrap();

                Some(ASTMetadata { resultant_tree: Self::RETURN(Some(ret_value)), remaining_slice: semicolon_idx.next_clone(), extra_stack_used: MemoryLayout::new() })
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
                    writeln!(result, "pop rax").unwrap();
                }
                //warning: ensure result is in the correct register and correctly sized
                write!(result, "{}", asm_boilerplate::func_exit_boilerplate()).unwrap();
            },
        }

        result
    }
}