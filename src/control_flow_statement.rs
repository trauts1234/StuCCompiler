use crate::{args_handling::location_allocation::{generate_only_return_location, ReturnLocation}, asm_gen_data::{AsmData, GlobalAsmData}, assembly::{assembly::Assembly, comparison::AsmComparison, operand::Storage, operation::{AsmOperation, CalleeReturnData}}, ast_metadata::ASTMetadata, data_type::{base_type::BaseType, recursive_data_type::DataType}, debugging::ASTDisplay, expression::expression::{self, promote, Expression}, generate_ir_traits::{GenerateIR, GetType}, lexer::{keywords::Keyword, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, parse_data::ParseData};
use colored::Colorize;
use stack_management::simple_stack_frame::SimpleStackFrame;

/**
 * this handles break, continue and return statements
 */
pub enum ControlFlowChange {
    RETURN(Option<Expression>),
    BREAK,
}

impl ControlFlowChange {
    pub fn try_consume(tokens_queue: &TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<ASTMetadata<ControlFlowChange>> {
        let mut curr_queue_idx = previous_queue_idx.clone();

        let kw = if let Some(Token::KEYWORD(x)) = tokens_queue.consume(&mut curr_queue_idx, &scope_data) {x} else {return None;};
        
        match kw {
            Keyword::RETURN => {

                //try to find semicolon at end of return statement
                let semicolon_idx = tokens_queue.find_closure_matches(&curr_queue_idx, false, |x| *x == Token::PUNCTUATOR(Punctuator::SEMICOLON), &TokenSearchType::skip_all_brackets())?;

                let return_value_slice = TokenQueueSlice{//between return statement and ; non inclusive
                    index: curr_queue_idx.index, 
                    max_index: semicolon_idx
                };

                let return_value = match return_value_slice.get_slice_size() {
                    0 => None,
                    1.. => Some(expression::try_consume_whole_expr(tokens_queue, &return_value_slice, scope_data).unwrap())
                };

                Some(ASTMetadata { resultant_tree: Self::RETURN(return_value), remaining_slice: TokenQueueSlice { index: semicolon_idx+1, max_index: curr_queue_idx.max_index } })
            }
            Keyword::BREAK => {
                assert!(tokens_queue.consume(&mut curr_queue_idx, scope_data) == Some(Token::PUNCTUATOR(Punctuator::SEMICOLON)));

                Some(ASTMetadata { remaining_slice: curr_queue_idx, resultant_tree: Self::BREAK})
            }
            _ => None
        }
    }
}

impl GenerateIR for ControlFlowChange {
    fn generate_ir(&self, asm_data: &AsmData, stack_data: &mut SimpleStackFrame, global_asm_data: &GlobalAsmData) -> (Assembly, Option<stack_management::stack_item::StackItemKey>) {
        let mut result = Assembly::make_empty();

        match self {
            ControlFlowChange::RETURN(expression) => {
                let return_data = 
                    expression
                    .as_ref()
                    .map(|expr| {
                        //find out what type to return and metadata about it
                        let return_type = asm_data.get_function_return_type();
                        let return_location = match generate_only_return_location(return_type, asm_data).unwrap() {
                            ReturnLocation::InRegs(eight_byte_locations) => CalleeReturnData::InRegs(eight_byte_locations),
                            ReturnLocation::HiddenPointer => todo!(),
                        };
                        //generate the return value
                        let (expr_asm, expr_location) = expr.generate_ir(asm_data, stack_data, global_asm_data);
                        result.merge(&expr_asm);
                        //cast to resultant type
                        let (cast_asm, cast_result) = promote(expr_location.unwrap(), expr.get_type(asm_data), return_type.clone(), stack_data, asm_data);
                        result.add_instruction(cast_asm);
                        //have the IR deal with where to put the result
                        (return_location, Storage::Stack(cast_result))
                    });

                //destroy stack frame and return
                result.add_instruction(AsmOperation::DestroyStackFrame);
                result.add_instruction(AsmOperation::Return {
                    return_data,
                });
            },
            ControlFlowChange::BREAK => {
                let label = asm_data.get_break_label().expect("break statement outside of a loop");
                //unconditionally jump to the label
                //signedness does not matter as it unconditionally jumps
                result.add_instruction(AsmOperation::JMPCC { label: label.clone(), comparison: AsmComparison::ALWAYS});
            },
        }

        (result, None)
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