use memory_size::MemoryLayout;

use crate::{asm_generation::asm_line, ast_metadata::ASTMetadata, compilation_state::{functions::FunctionList, stack_variables::StackVariables}, expression::{self, ExprNode}, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size};
use std::fmt::Write;

/**
 * this handles break, continue and return statements
 */
pub enum ControlFlowChange {
    RETURN(Option<Box<dyn ExprNode>>)
}

impl ControlFlowChange {
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, local_variables: &StackVariables, accessible_funcs: &FunctionList) -> Option<ASTMetadata<ControlFlowChange>> {
        let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        let kw = if let Some(Token::KEYWORD(x)) = tokens_queue.consume(&mut curr_queue_idx) {x} else {return None;};
        
        match kw.as_str() {
            "return" => {

                //try to find semicolon at end of return statement
                let semicolon_idx = tokens_queue.find_closure_in_slice(&curr_queue_idx, false, |x| *x == Token::PUNCTUATOR(Punctuator::SEMICOLON))?;

                let return_value_slice = TokenQueueSlice{//between return statement and ; non inclusive
                    index: curr_queue_idx.index, 
                    max_index: semicolon_idx.index
                };

                if return_value_slice.get_slice_size() == 0 {
                    //func returned nothing(void)
                    //return slice of all tokens after the semicolon
                    return Some(ASTMetadata{resultant_tree: Self::RETURN(None),remaining_slice: semicolon_idx.next_clone(), extra_stack_used: MemoryLayout::new()});
                }

                //try and match with an expression for what to return
                let ret_value = expression::try_consume_whole_expr(tokens_queue, &return_value_slice, local_variables, accessible_funcs).unwrap();

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
                    asm_line!(result, "{}", expr.generate_assembly());

                    //asm_line!(result, "{}", asm_boilerplate::cast_from_acc(&expr.get_data_type(), todo!()));
                    //TODO cast here - how do I know the function's return type???
                }
                //warning: ensure result is in the correct register and correctly sized
                //destroy stack frame and return
                asm_line!(result, "mov rsp, rbp");
                asm_line!(result, "pop rbp");
                asm_line!(result, "ret");
            },
        }

        result
    }
}