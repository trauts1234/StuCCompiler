use crate::{asm_gen_data::AsmData, asm_generation::{asm_line, LogicalRegister, RegisterName}, ast_metadata::ASTMetadata, block_statement::StatementOrDeclaration, compilation_state::{functions::FunctionList, label_generator::LabelGenerator}, expression::{self, ExprNode}, lexer::{keywords::Keyword, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, parse_data::ParseData, statement::Statement};
use std::fmt::Write;

/**
 * this handles if statements and other conditionals
 */
pub enum IterationStatement{
    FOR{
        initialisation: Box<StatementOrDeclaration>,//can't be anything fancy like a scope or if statement, but expressions and declarations are OK
        condition: Box<dyn ExprNode>,
        increment: Option<Box<dyn ExprNode>>,

        local_scope_data: ParseData,//metadata to help with assembly generation

        body: Box<Statement>
    },
    WHILE {
        condition: Box<dyn ExprNode>,
        body: Box<Statement>,

        local_scope_data: ParseData//do I need this for a while loop?
    }
}

impl IterationStatement {
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, outer_scope_data: &ParseData) -> Option<ASTMetadata<IterationStatement>> {
        let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        let kw = if let Some(Token::KEYWORD(x)) = tokens_queue.consume(&mut curr_queue_idx, outer_scope_data) {x} else {return None;};

        //important: clone the local variables and enums, to prevent inner definitions from leaking out to outer scopes
        let mut in_loop_data = outer_scope_data.clone_for_new_scope();
        
        match kw {
            Keyword::FOR => {
                let closecurly_idx = tokens_queue.find_matching_close_bracket(curr_queue_idx.index);
                assert!(Token::PUNCTUATOR(Punctuator::OPENCURLY) == tokens_queue.consume(&mut curr_queue_idx, &in_loop_data).unwrap());//ensure opening parenthesis

                let items_slice = TokenQueueSlice{
                    index: curr_queue_idx.index,
                    max_index: closecurly_idx
                };

                let items = tokens_queue.split_outside_parentheses(&items_slice, |x| *x == Token::PUNCTUATOR(Punctuator::SEMICOLON));
                assert!(items.len() == 3);
                let (init_slice, condition_slice, increment_slice) = (&items[0], &items[1], &items[2]);//get the slices that I need

                let init_with_semicolon = TokenQueueSlice {
                    index:init_slice.index,
                    max_index:init_slice.max_index+1
                };

                let ASTMetadata {resultant_tree:init, extra_stack_used:init_stack_used, .. } = StatementOrDeclaration::try_consume(tokens_queue, &init_with_semicolon, accessible_funcs, &mut in_loop_data).unwrap();
                let condition = expression::try_consume_whole_expr(tokens_queue, &condition_slice, accessible_funcs, &mut in_loop_data).unwrap();
                let increment = expression::try_consume_whole_expr(tokens_queue, &increment_slice, accessible_funcs, &mut in_loop_data);

                //consume the "for (;;)" part
                curr_queue_idx = TokenQueueSlice{
                    index: closecurly_idx + 1,
                    max_index: curr_queue_idx.max_index
                };

                //consume the body
                let ASTMetadata{ remaining_slice, resultant_tree: loop_body, extra_stack_used:body_stack_used } = Statement::try_consume(tokens_queue, &curr_queue_idx, accessible_funcs, &mut in_loop_data).unwrap();
                curr_queue_idx = remaining_slice;

                let extra_stack_used = body_stack_used + init_stack_used;//includes iterator variable

                Some(ASTMetadata{
                    resultant_tree: Self::FOR { initialisation: Box::new(init), condition: condition, increment: increment, body: Box::new(loop_body), local_scope_data: in_loop_data }, 
                    remaining_slice: curr_queue_idx, 
                    extra_stack_used: extra_stack_used})
            },
            Keyword::WHILE => {
                let closecurly_idx = tokens_queue.find_matching_close_bracket(curr_queue_idx.index);
                assert!(Token::PUNCTUATOR(Punctuator::OPENCURLY) == tokens_queue.consume(&mut curr_queue_idx, &in_loop_data).unwrap());//ensure opening parenthesis

                let condition_slice = TokenQueueSlice{
                    index: curr_queue_idx.index,
                    max_index: closecurly_idx
                };

                let condition = expression::try_consume_whole_expr(tokens_queue, &condition_slice, accessible_funcs, &mut in_loop_data).unwrap();

                //consume the "while ()" part
                curr_queue_idx = TokenQueueSlice{
                    index: closecurly_idx + 1,
                    max_index: curr_queue_idx.max_index
                };

                //consume the body
                let ASTMetadata{ remaining_slice, resultant_tree: loop_body, extra_stack_used:body_stack_used } = Statement::try_consume(tokens_queue, &curr_queue_idx, accessible_funcs, &mut in_loop_data).unwrap();
                curr_queue_idx = remaining_slice;

                Some(ASTMetadata{
                    resultant_tree: Self::WHILE { condition: condition, body: Box::new(loop_body), local_scope_data: in_loop_data  }, 
                    remaining_slice: curr_queue_idx, 
                    extra_stack_used: body_stack_used})
            }
            _ => None
        }
    }

    pub fn generate_assembly(&self, label_gen: &mut LabelGenerator, asm_data: &AsmData) -> String {
        let mut result = String::new();

        match self {
            Self::FOR { initialisation, condition, increment, body, local_scope_data } => {

                let asm_data = &asm_data.clone_for_new_scope(local_scope_data, asm_data.get_function_return_type().clone());
                
                let condition_size = &condition.get_data_type(asm_data).memory_size();
                assert!(condition.get_data_type(asm_data).underlying_type().is_integer());//cmp 0 may not work for float. but may work for pointers????

                let generic_label = label_gen.generate_label();

                asm_line!(result, "{}", initialisation.generate_assembly(label_gen, asm_data));//initialise the for loop anyways

                asm_line!(result, "{}_loop_start:", generic_label);//label for loop's start

                asm_line!(result, "{}", condition.generate_assembly(asm_data));//generate the condition

                asm_line!(result, "cmp {}, 0", LogicalRegister::ACC.generate_reg_name(condition_size));//compare the result to 0
                asm_line!(result, "je {}_loop_end", generic_label);//if the result is 0, jump to the end of the loop

                asm_line!(result, "{}", body.generate_assembly(label_gen, asm_data));//generate the loop body

                asm_line!(result, "{}_loop_increment:", generic_label);//add label to jump to incrementing the loop

                if let Some(inc) = increment {//if there is an increment
                    asm_line!(result, "{}", inc.generate_assembly(asm_data));//apply the increment
                }
                asm_line!(result, "jmp {}_loop_start", generic_label);//after increment, go to top of loop

                asm_line!(result, "{}_loop_end:", generic_label);
            },

            Self::WHILE { condition, body, local_scope_data } => {

                let asm_data = &asm_data.clone_for_new_scope(local_scope_data, asm_data.get_function_return_type().clone());

                let condition_size = &condition.get_data_type(asm_data).memory_size();

                let generic_label = label_gen.generate_label();

                asm_line!(result, "{}_loop_start:", generic_label);//label for loop's start

                asm_line!(result, "{}", condition.generate_assembly(asm_data));//generate the condition

                assert!(condition.get_data_type(asm_data).underlying_type().is_integer());//cmp 0 may not work for float. but may work for pointers????

                asm_line!(result, "cmp {}, 0", LogicalRegister::ACC.generate_reg_name(condition_size));//compare the result to 0
                asm_line!(result, "je {}_loop_end", generic_label);//if the result is 0, jump to the end of the loop

                asm_line!(result, "{}", body.generate_assembly(label_gen, asm_data));//generate the loop body

                asm_line!(result, "jmp {}_loop_start", generic_label);//after loop complete, go to top of loop

                asm_line!(result, "{}_loop_end:", generic_label);
            }
        }

        result
    }
}