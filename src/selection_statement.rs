use crate::{asm_generation::{asm_line, LogicalRegister, RegisterName}, ast_metadata::ASTMetadata, compilation_state::{functions::FunctionList, label_generator::LabelGenerator, stack_variables::StackVariables}, expression::Expression, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size::MemoryLayout, statement::Statement};
use std::fmt::Write;

/**
 * this handles if statements and other conditionals
 */
#[derive(Debug)]
pub enum SelectionStatement{
    IF{
        condition: Expression,
        if_body: Box<Statement>,
        else_body: Option<Box<Statement>>
    }
}

impl SelectionStatement {
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, local_variables: &StackVariables, accessible_funcs: &FunctionList) -> Option<ASTMetadata<SelectionStatement>> {
        let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        let kw = if let Some(Token::KEYWORD(x)) = tokens_queue.consume(&mut curr_queue_idx) {x} else {return None;};
        
        match kw.as_str() {
            "if" => {
                assert!(Token::PUNCTUATOR(Punctuator::OPENCURLY) == tokens_queue.consume(&mut curr_queue_idx).unwrap());//ensure opening parenthesis
                
                let closecurly_idx = tokens_queue.find_closure_in_slice(&curr_queue_idx, false, |x| *x == Token::PUNCTUATOR(Punctuator::CLOSECURLY)).unwrap();

                let condition_slice = TokenQueueSlice{
                    index: curr_queue_idx.index,
                    max_index: closecurly_idx.index
                };

                let condition = Expression::try_consume_whole_expr(tokens_queue, &condition_slice, local_variables, accessible_funcs).unwrap();

                //consume the condition
                curr_queue_idx = TokenQueueSlice{
                    index: closecurly_idx.index + 1,
                    max_index: curr_queue_idx.max_index
                };

                //consume the function body
                let ASTMetadata{ remaining_slice, resultant_tree: taken_body, .. } = Statement::try_consume(tokens_queue, &curr_queue_idx, local_variables, accessible_funcs).unwrap();
                curr_queue_idx = remaining_slice;

                let has_else_branch = tokens_queue.peek(&curr_queue_idx).is_some_and(|x| x == Token::KEYWORD("else".to_string()));

                //try and consume the else branch
                let not_taken_body: Option<Box<Statement>> = if has_else_branch {
                    tokens_queue.consume(&mut curr_queue_idx);//consume the else keyword
                    let ASTMetadata{ remaining_slice, resultant_tree: else_body, .. } = Statement::try_consume(tokens_queue, &curr_queue_idx, local_variables, accessible_funcs).unwrap();
                    curr_queue_idx = remaining_slice;//consume the else
                    Some(Box::new(else_body))
                } else {
                    None//no else branch
                };

                Some(ASTMetadata{
                    resultant_tree: Self::IF{condition, if_body: Box::new(taken_body), else_body: not_taken_body}, 
                    remaining_slice: curr_queue_idx, 
                    extra_stack_used: MemoryLayout::new()})
            }
            _ => None
        }
    }

    pub fn generate_assembly(&self, label_gen: &mut LabelGenerator) -> String {
        let mut result = String::new();

        match self {
            Self::IF { condition, if_body, else_body } => {
                let generic_label = label_gen.generate_label();
                let else_label = format!("{}_else", generic_label);//jump for the else branch
                let if_end_label = format!("{}_end", generic_label);//rendevous point for the if and else branches

                let cond_false_label = if else_body.is_some() {&else_label} else {&if_end_label};

                asm_line!(result, "{}", condition.generate_assembly());//generate the condition to acc
                
                let condition_size = &condition.get_data_type().memory_size();

                assert!(condition.get_data_type().underlying_type_is_integer());//cmp 0 may not work for float. but may work for pointers????
  
                asm_line!(result, "cmp {}, 0", LogicalRegister::ACC.generate_reg_name(condition_size));//compare the result to 0
                asm_line!(result, "je {}", cond_false_label);//if the result is 0, jump to the else block or the end of the if statement

                asm_line!(result, "{}", if_body.generate_assembly(label_gen));//generate the body of the if statement
                asm_line!(result, "jmp {}", if_end_label);//jump to the end of the if/else block

                if let Some(else_body) = else_body {
                    //there is code in the else block
                    asm_line!(result, "{}:", else_label);//start of the else block
                    asm_line!(result, "{}", else_body.generate_assembly(label_gen));//generate the body of the else statement
                }

                asm_line!(result, "{}:", if_end_label);//after if/else are complete, jump here

            }
        }

        result
    }
}