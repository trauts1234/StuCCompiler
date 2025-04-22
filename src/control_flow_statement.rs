use crate::{asm_boilerplate::cast_from_acc, asm_gen_data::AsmData, assembly::{assembly::Assembly, operation::{AsmComparison, AsmOperation}}, ast_metadata::ASTMetadata, compilation_state::functions::FunctionList, data_type::{base_type::BaseType, recursive_data_type::DataType}, debugging::ASTDisplay, expression::expression::{self, Expression}, expression_visitors::{data_type_visitor::GetDataTypeVisitor, put_scalar_in_acc::ScalarInAccVisitor}, lexer::{keywords::Keyword, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, parse_data::ParseData};
use colored::Colorize;
use memory_size::MemorySize;

/**
 * this handles break, continue and return statements
 */
pub enum ControlFlowChange {
    RETURN(Option<Expression>),
    BREAK,
}

impl ControlFlowChange {
    pub fn try_consume(tokens_queue: &TokenQueue, previous_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, scope_data: &mut ParseData) -> Option<ASTMetadata<ControlFlowChange>> {
        let mut curr_queue_idx = previous_queue_idx.clone();

        let kw = if let Some(Token::KEYWORD(x)) = tokens_queue.consume(&mut curr_queue_idx, &scope_data) {x} else {return None;};
        
        match kw {
            Keyword::RETURN => {

                //try to find semicolon at end of return statement
                let semicolon_idx = tokens_queue.find_closure_matches(&curr_queue_idx, false, |x| *x == Token::PUNCTUATOR(Punctuator::SEMICOLON), &TokenSearchType::skip_all())?;

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
            Keyword::BREAK => {
                assert!(tokens_queue.consume(&mut curr_queue_idx, scope_data) == Some(Token::PUNCTUATOR(Punctuator::SEMICOLON)));

                Some(ASTMetadata { remaining_slice: curr_queue_idx, resultant_tree: Self::BREAK})
            }
            _ => None
        }
    }

    pub fn generate_assembly(&self, asm_data: &AsmData, stack_data: &mut MemorySize) -> Assembly {
        let mut result = Assembly::make_empty();

        match self {
            ControlFlowChange::RETURN(expression) => {
                if let Some(expr) = expression {
                    let expr_asm = expr.accept(&mut ScalarInAccVisitor {asm_data, stack_data});
                    result.merge(&expr_asm);

                    match expr.accept(&mut GetDataTypeVisitor{asm_data}) {
                        DataType::ARRAY {..} => panic!("tried to return array from function!"),
                        expr_type => {
                            if let DataType::RAW(BaseType::STRUCT(struct_name)) = expr_type {
                                todo!("returning struct {:?} from function", struct_name)
                            } else {
                                let cast_asm = cast_from_acc(&expr_type, asm_data.get_function_return_type(), asm_data);
                                result.merge(&cast_asm);
                            }
                        },
                    }

                }
                //destroy stack frame and return
                result.add_instruction(AsmOperation::DestroyStackFrame);
                result.add_instruction(AsmOperation::Return);
            },
            ControlFlowChange::BREAK => {
                let label = asm_data.get_break_label().expect("break statement outside of a loop");
                //unconditionally jump to the label
                //signedness does not matter as it unconditionally jumps
                result.add_instruction(AsmOperation::JMPCC { label: label.clone(), comparison: AsmComparison::ALWAYS, signed_comparison: true });
            },
        }

        result
    }
}

impl ASTDisplay for ControlFlowChange {
    fn display_ast(&self, f: &mut crate::debugging::TreeDisplayInfo) {
        match self {
            ControlFlowChange::RETURN(expression) => {
                f.write(&"return".yellow());
                if let Some(expr) = expression {
                    f.indent();
                    expr.display_ast(f);
                    f.dedent();
                }
            }
            ControlFlowChange::BREAK => f.write(&"break".yellow()),
        }
    }
}