use crate::{ast_metadata::ASTMetadata, expression::Expression, label_generator::LabelGenerator, lexer::{token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue, Punctuator::Punctuator}, memory_size::MemoryLayout, stack_variables::StackVariables, statement::Statement};
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
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, local_variables: &StackVariables) -> Option<ASTMetadata<SelectionStatement>> {
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

                let condition = Expression::try_consume_whole_expr(tokens_queue, &condition_slice, local_variables).unwrap();

                //consume the condition
                curr_queue_idx = TokenQueueSlice{
                    index: closecurly_idx.index + 1,
                    max_index: curr_queue_idx.max_index
                };

                //consume the function body
                let ASTMetadata{ remaining_slice, resultant_tree: taken_body, .. } = Statement::try_consume(tokens_queue, &curr_queue_idx, local_variables).unwrap();
                curr_queue_idx = remaining_slice;

                let has_else_branch = tokens_queue.peek(&curr_queue_idx).is_some_and(|x| x == Token::KEYWORD("else".to_string()));

                //try and consume the else branch
                let not_taken_body: Option<Box<Statement>> = if has_else_branch {
                    tokens_queue.consume(&mut curr_queue_idx);//consume the else keyword
                    let ASTMetadata{ remaining_slice, resultant_tree: else_body, .. } = Statement::try_consume(tokens_queue, &curr_queue_idx, local_variables).unwrap();
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
                let if_end_label = format!("{}_not_taken", generic_label);//rendevous point for the if and else branches

                write!(result, "{}", condition.generate_assembly()).unwrap();//generate the condition
                writeln!(result, "pop rax").unwrap();
                writeln!(result, "cmp rax, 0").unwrap();//compare the result to 0
                writeln!(result, "je {}", if_end_label).unwrap();//if the result is 0, jump to the else block

                write!(result, "{}", if_body.generate_assembly(label_gen)).unwrap();//generate the body of the if statement
                writeln!(result, "jmp {}", if_end_label).unwrap();//jump to the end of the if/else block

                if let Some(else_body) = else_body {
                    //there is code in the else block
                    writeln!(result, "{}:", else_label).unwrap();//start of the else block
                    write!(result, "{}", else_body.generate_assembly(label_gen)).unwrap();//generate the body of the else statement
                }

                writeln!(result, "{}:", if_end_label).unwrap();//after if/else are complete, jump here

            }
        }

        result
    }
}