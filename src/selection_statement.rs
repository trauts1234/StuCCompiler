use crate::{asm_gen_data::AsmData, asm_generation::{asm_line, LogicalRegister, RegisterName}, ast_metadata::ASTMetadata, compilation_state::{functions::FunctionList, label_generator::LabelGenerator}, expression::{self, Expression}, expression_visitors::{data_type_visitor::GetDataTypeVisitor, put_scalar_in_acc::ScalarInAccVisitor}, lexer::{keywords::Keyword, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size::MemoryLayout, parse_data::ParseData, statement::Statement};
use std::fmt::Write;

/**
 * this handles if statements and other conditionals
 */
pub enum SelectionStatement{
    IF{
        condition: Expression,
        if_body: Box<Statement>,
        else_body: Option<Box<Statement>>
    }
}

impl SelectionStatement {
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, scope_data: &ParseData) -> Option<ASTMetadata<SelectionStatement>> {
        let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        let kw = if let Some(Token::KEYWORD(x)) = tokens_queue.consume(&mut curr_queue_idx, &scope_data) {x} else {return None;};

        //scope_data is not cloned here as the scope will clone it for us
        
        match kw {
            Keyword::IF => {
                
                let closecurly_idx = tokens_queue.find_matching_close_bracket(curr_queue_idx.index);
                assert!(Token::PUNCTUATOR(Punctuator::OPENCURLY) == tokens_queue.consume(&mut curr_queue_idx, &scope_data).unwrap());//ensure opening parenthesis

                let condition_slice = TokenQueueSlice{
                    index: curr_queue_idx.index,
                    max_index: closecurly_idx
                };

                let condition = expression::try_consume_whole_expr(tokens_queue, &condition_slice, accessible_funcs, scope_data).unwrap();

                //consume the condition
                curr_queue_idx = TokenQueueSlice{
                    index: closecurly_idx + 1,
                    max_index: curr_queue_idx.max_index
                };

                //consume the function body
                let ASTMetadata{ remaining_slice, resultant_tree: taken_body } = Statement::try_consume(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data).unwrap();
                curr_queue_idx = remaining_slice;

                let has_else_branch = tokens_queue.peek(&curr_queue_idx, &scope_data).is_some_and(|x| x == Token::KEYWORD(Keyword::ELSE));

                //try and consume the else branch
                let not_taken_body: Option<Box<Statement>> = if has_else_branch {
                    tokens_queue.consume(&mut curr_queue_idx, &scope_data);//consume the else keyword
                    let ASTMetadata{ remaining_slice, resultant_tree: else_body} = Statement::try_consume(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data).unwrap();
                    curr_queue_idx = remaining_slice;//consume the else

                    Some(Box::new(else_body))
                } else {
                    None//no else branch
                };

                Some(ASTMetadata{
                    resultant_tree: Self::IF{condition, if_body: Box::new(taken_body), else_body: not_taken_body}, 
                    remaining_slice: curr_queue_idx, 
                })
            }
            _ => None
        }
    }

    pub fn generate_assembly(&self, label_gen: &mut LabelGenerator, asm_data: &AsmData) -> String {
        let mut result = String::new();

        match self {
            Self::IF { condition, if_body, else_body } => {
                let generic_label = label_gen.generate_label();
                let else_label = format!("{}_else", generic_label);//jump for the else branch
                let if_end_label = format!("{}_end", generic_label);//rendevous point for the if and else branches

                let cond_false_label = if else_body.is_some() {&else_label} else {&if_end_label};

                asm_line!(result, "{}", condition.accept(&mut ScalarInAccVisitor {asm_data}));//generate the condition to acc
                
                let condition_type = condition.accept(&mut GetDataTypeVisitor {asm_data});

                let condition_size = &condition_type.memory_size(asm_data);
  
                asm_line!(result, "cmp {}, 0", LogicalRegister::ACC.generate_reg_name(condition_size));//compare the result to 0
                asm_line!(result, "je {}", cond_false_label);//if the result is 0, jump to the else block or the end of the if statement

                asm_line!(result, "{}", if_body.generate_assembly(label_gen, asm_data));//generate the body of the if statement
                asm_line!(result, "jmp {}", if_end_label);//jump to the end of the if/else block

                if let Some(else_body) = else_body {
                    //there is code in the else block
                    asm_line!(result, "{}:", else_label);//start of the else block
                    asm_line!(result, "{}", else_body.generate_assembly(label_gen, asm_data));//generate the body of the else statement
                }

                asm_line!(result, "{}:", if_end_label);//after if/else are complete, jump here

            }
        }

        result
    }

    pub fn get_stack_height(&self, asm_data: &AsmData) -> Option<MemoryLayout> {
        match self {
            SelectionStatement::IF { condition: _, if_body, else_body } =>  {
                let possible_else_size = else_body.as_ref().and_then(|x| x.get_stack_height(asm_data));

                match (if_body.get_stack_height(asm_data), possible_else_size) {
                    (Some(x), Some(y)) => Some(MemoryLayout::biggest(&x, &y)),//only one of (if, else) ever gets run, so I only need enough stack for the biggest of the two
                    (x, None) => x,//no else, so just need enough stack for the if branch
                    (None, y) => y//this doesn't mean that there is no if body, it just means there is no useful data there
                }
            },
        }
    }
}