use crate::{asm_gen_data::{AsmData, GlobalAsmData}, assembly::{assembly::Assembly, comparison::AsmComparison, operand::{immediate::ImmediateValue, register::GPRegister, Operand}, operation::{AsmOperation, Label}}, ast_metadata::ASTMetadata, compilation_state::label_generator::LabelGenerator, data_type::{base_type::BaseType, recursive_data_type::DataType}, debugging::ASTDisplay, expression::expression::{self, Expression}, expression_visitors::{data_type_visitor::GetDataTypeVisitor, put_scalar_in_acc::ScalarInAccVisitor}, lexer::{keywords::Keyword, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, parse_data::ParseData, stack_allocation::StackAllocator, statement::Statement};
use colored::Colorize;
use memory_size::MemorySize;
use unwrap_let::unwrap_let;

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
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData, struct_label_gen: &mut LabelGenerator) -> Option<ASTMetadata<SelectionStatement>> {
        let mut curr_queue_idx = previous_queue_idx.clone();

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

                let condition = expression::try_consume_whole_expr(tokens_queue, &condition_slice, scope_data, struct_label_gen).expect(&tokens_queue.display_slice(&condition_slice));

                //consume the condition
                curr_queue_idx = TokenQueueSlice{
                    index: closecurly_idx + 1,
                    max_index: curr_queue_idx.max_index
                };

                //consume the function body
                let ASTMetadata{ remaining_slice, resultant_tree: taken_body } = Statement::try_consume(tokens_queue, &curr_queue_idx, scope_data, struct_label_gen).unwrap();
                curr_queue_idx = remaining_slice;

                let has_else_branch = tokens_queue.peek(&curr_queue_idx, &scope_data).is_some_and(|x| x == Token::KEYWORD(Keyword::ELSE));

                //try and consume the else branch
                let not_taken_body: Option<Box<Statement>> = if has_else_branch {
                    tokens_queue.consume(&mut curr_queue_idx, &scope_data);//consume the else keyword
                    let ASTMetadata{ remaining_slice, resultant_tree: else_body} = Statement::try_consume(tokens_queue, &curr_queue_idx, scope_data, struct_label_gen).unwrap();
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

    pub fn generate_assembly(&self, asm_data: &AsmData, stack_data: &mut StackAllocator, global_asm_data: &mut GlobalAsmData) -> Assembly {
        let mut result = Assembly::make_empty();

        match self {
            Self::IF { condition, if_body, else_body } => {
                let generic_label = global_asm_data.label_gen_mut().generate_label();
                let else_label = Label::Local(format!("{}_else", generic_label));//jump for the else branch
                let if_end_label = Label::Local(format!("{}_end", generic_label));//rendevous point for the if and else branches

                let cond_false_label = if else_body.is_some() {&else_label} else {&if_end_label};//only jump to else branch if it exists

                let condition_asm = condition.accept(&mut ScalarInAccVisitor {asm_data, stack_data});
                result.merge(&condition_asm);//generate the condition to acc
                
                unwrap_let!(DataType::RAW(BaseType::Scalar(condition_type)) = condition.accept(&mut GetDataTypeVisitor {asm_data}));

                //compare the result to 0
                result.add_instruction(AsmOperation::CMP {
                    rhs: Operand::Imm(ImmediateValue("0".to_string())),
                    data_type: condition_type
                });

                //if the result is 0, jump to the else block or the end of the if statement
                result.add_instruction(AsmOperation::JMPCC {
                    label: cond_false_label.clone(),
                    comparison: AsmComparison::EQ,
                });

                let (mut if_body_stack_usage, mut else_body_stack_usage) = stack_data.split_for_branching();

                //generate the body of the if statement
                let if_body_asm = if_body.generate_assembly(asm_data, &mut if_body_stack_usage, global_asm_data);
                result.merge(&if_body_asm);

                //jump to the end of the if/else block
                result.add_instruction(AsmOperation::JMPCC {
                    label: if_end_label.clone(),
                    comparison: AsmComparison::ALWAYS,//unconditional jump
                });

                if let Some(else_body) = else_body {
                    //there is code in the else block

                    let else_body_asm = else_body.generate_assembly(asm_data, &mut else_body_stack_usage, global_asm_data);

                    //start of the else block
                    result.add_instruction(AsmOperation::Label(else_label));//add label
                    result.merge(&else_body_asm);//generate the body of the else statement
                }

                //stack required is the largest between the if and else branches
                stack_data.merge_from_branching(if_body_stack_usage, else_body_stack_usage);

                //after if/else are complete, jump here
                result.add_instruction(AsmOperation::Label(if_end_label));

            }
        }

        result
    }
}

impl ASTDisplay for SelectionStatement {
    fn display_ast(&self, f: &mut crate::debugging::TreeDisplayInfo) {
        match self {
            SelectionStatement::IF { condition, if_body, else_body } => {
                f.write(&"if statement".red().to_string());
                f.indent();

                f.write(&"condition".green().to_string());
                f.indent();
                condition.display_ast(f);
                f.dedent();

                f.write(&"if-body".red().to_string());
                f.indent();
                if_body.display_ast(f);
                f.dedent();

                if let Some(else_body) = else_body {
                    f.write(&"else-body".red().to_string());
                    f.indent();
                    else_body.display_ast(f);
                    f.dedent();
                }

                f.dedent();
            }
        }
    }
}