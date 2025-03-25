use unwrap_let::unwrap_let;

use crate::{asm_boilerplate, asm_gen_data::AsmData, asm_generation::asm_line, ast_metadata::ASTMetadata, compilation_state::functions::FunctionList, data_type::data_type::DataType, expression::{self, Expression}, expression_visitors::{data_type_visitor::GetDataTypeVisitor, put_scalar_in_acc::ScalarInAccVisitor}, lexer::{keywords::Keyword, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, parse_data::ParseData};
use std::fmt::Write;

/**
 * this handles break, continue and return statements
 */
pub enum ControlFlowChange {
    RETURN(Option<Expression>)
}

impl ControlFlowChange {
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, scope_data: &ParseData) -> Option<ASTMetadata<ControlFlowChange>> {
        let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        let kw = if let Some(Token::KEYWORD(x)) = tokens_queue.consume(&mut curr_queue_idx, &scope_data) {x} else {return None;};
        
        match kw {
            Keyword::RETURN => {

                //try to find semicolon at end of return statement
                let semicolon_idx = tokens_queue.find_closure_matches(&curr_queue_idx, false, |x| *x == Token::PUNCTUATOR(Punctuator::SEMICOLON), &TokenSearchType::skip_nothing())?;

                let return_value_slice = TokenQueueSlice{//between return statement and ; non inclusive
                    index: curr_queue_idx.index, 
                    max_index: semicolon_idx.index
                };

                let return_value = match return_value_slice.get_slice_size() {
                    0 => None,
                    1.. => Some(expression::try_consume_whole_expr(tokens_queue, &return_value_slice, accessible_funcs, scope_data).unwrap())
                };

                Some(ASTMetadata { resultant_tree: Self::RETURN(return_value), remaining_slice: semicolon_idx.next_clone() })
            }
            _ => None
        }
    }

    pub fn generate_assembly(&self, asm_data: &AsmData) -> String {
        let mut result = String::new();

        match self {
            ControlFlowChange::RETURN(expression) => {
                if let Some(expr) = expression {
                    asm_line!(result, "{}", expr.accept(&mut ScalarInAccVisitor {asm_data}));

                    match expr.accept(&mut GetDataTypeVisitor {asm_data}) {
                        DataType::PRIMATIVE(primative) => { 
                            unwrap_let!(DataType::PRIMATIVE(ret_type_primative) = asm_data.get_function_return_type());
                            asm_line!(result, "{}", asm_boilerplate::cast_from_acc(&primative, ret_type_primative))
                        },
                        DataType::COMPOSITE(composite) => todo!("returning composite value from function"),
                    }

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