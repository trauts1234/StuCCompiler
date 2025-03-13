use memory_size::MemoryLayout;

use crate::{asm_boilerplate, asm_generation::asm_line, ast_metadata::ASTMetadata, compilation_state::functions::FunctionList, data_type::data_type::DataType, expression::{self, ExprNode}, lexer::{keywords::Keyword, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size, scope_data::ScopeData};
use std::fmt::Write;

pub struct ReturnValue {
    expr: Box<dyn ExprNode>,
    function_ret_type: DataType
}

/**
 * this handles break, continue and return statements
 */
pub enum ControlFlowChange {
    RETURN(Option<ReturnValue>)
}

impl ControlFlowChange {
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, scope_data: &mut ScopeData) -> Option<ASTMetadata<ControlFlowChange>> {
        let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        let kw = if let Some(Token::KEYWORD(x)) = tokens_queue.consume(&mut curr_queue_idx) {x} else {return None;};
        
        match kw {
            Keyword::RETURN => {

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
                let ret_value = ReturnValue { 
                    expr: expression::try_consume_whole_expr(tokens_queue, &return_value_slice, accessible_funcs, scope_data).unwrap(),
                    function_ret_type: scope_data.stack_vars.get_return_type().clone()
                };

                Some(ASTMetadata { resultant_tree: Self::RETURN(Some(ret_value)), remaining_slice: semicolon_idx.next_clone(), extra_stack_used: MemoryLayout::new() })
            }
            _ => None
        }
    }

    pub fn generate_assembly(&self) -> String {
        let mut result = String::new();

        match self {
            ControlFlowChange::RETURN(expression) => {
                if let Some(ReturnValue { expr, function_ret_type }) = expression {
                    asm_line!(result, "{}", expr.generate_assembly());

                    asm_line!(result, "{}", asm_boilerplate::cast_from_acc(&expr.get_data_type(), function_ret_type));
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