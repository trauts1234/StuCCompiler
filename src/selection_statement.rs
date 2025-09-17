use crate::{asm_gen_data::{AsmData, GlobalAsmData}, assembly::{assembly::Assembly, comparison::AsmComparison, operand::Storage, operation::{AsmOperation, Label}}, ast_metadata::ASTMetadata, data_type::{base_type::{BaseType, ScalarType}, recursive_data_type::DataType}, debugging::ASTDisplay, expression::expression::{self, Expression}, expression_visitors::data_type_visitor::GetDataTypeVisitor, lexer::{keywords::Keyword, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, number_literal::typed_value::NumberLiteral, parse_data::ParseData, generate_ir::GenerateIR, statement::Statement};
use colored::Colorize;
use stack_management::{simple_stack_frame::SimpleStackFrame, stack_item::StackItemKey};
use unwrap_let::unwrap_let;
use uuid::Uuid;

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
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<ASTMetadata<SelectionStatement>> {
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

                let condition = expression::try_consume_whole_expr(tokens_queue, &condition_slice, scope_data).expect(&tokens_queue.display_slice(&condition_slice));

                //consume the condition
                curr_queue_idx = TokenQueueSlice{
                    index: closecurly_idx + 1,
                    max_index: curr_queue_idx.max_index
                };

                //consume the function body
                let ASTMetadata{ remaining_slice, resultant_tree: taken_body } = Statement::try_consume(tokens_queue, &curr_queue_idx, scope_data).unwrap();
                curr_queue_idx = remaining_slice;

                let has_else_branch = tokens_queue.peek(&curr_queue_idx, &scope_data).is_some_and(|x| x == Token::KEYWORD(Keyword::ELSE));

                //try and consume the else branch
                let not_taken_body: Option<Box<Statement>> = if has_else_branch {
                    tokens_queue.consume(&mut curr_queue_idx, &scope_data);//consume the else keyword
                    let ASTMetadata{ remaining_slice, resultant_tree: else_body} = Statement::try_consume(tokens_queue, &curr_queue_idx, scope_data).unwrap();
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
}

impl GenerateIR for SelectionStatement {
    fn generate_ir(&self, asm_data: &AsmData, stack_data: &mut SimpleStackFrame, global_asm_data: &GlobalAsmData) -> (Assembly, Option<StackItemKey>) {
        let mut result = Assembly::make_empty();

        match self {
            Self::IF { condition, if_body, else_body } => {
                let generic_label = Uuid::new_v4().simple().to_string();
                let else_label = Label::Local(format!("{}_else", generic_label));//jump for the else branch
                let if_end_label = Label::Local(format!("{}_end", generic_label));//rendevous point for the if and else branches

                let cond_false_label = if else_body.is_some() {&else_label} else {&if_end_label};//only jump to else branch if it exists

                let (condition_asm, condition_value) = condition.generate_ir(asm_data, stack_data, global_asm_data);
                result.merge(&condition_asm);//generate the condition to acc
                
                unwrap_let!(DataType::RAW(BaseType::Scalar(condition_type)) = condition.accept(&mut GetDataTypeVisitor {asm_data}));
                let zero = match condition_type {
                    ScalarType::Float(float_type) => NumberLiteral::FLOAT { data: 0f64, data_type: float_type },
                    ScalarType::Integer(integer_type) => NumberLiteral::INTEGER { data: 0, data_type: integer_type },
                };

                //compare the result to 0
                result.add_instruction(AsmOperation::CMP {
                    lhs: Storage::Stack(condition_value.unwrap()),
                    rhs: Storage::Constant(zero),
                    data_type: condition_type
                });

                //if the result is 0, jump to the else block or the end of the if statement
                result.add_instruction(AsmOperation::JMPCC {
                    label: cond_false_label.clone(),
                    comparison: AsmComparison::EQ,
                });

                //generate the body of the if statement
                let (if_body_asm, _) = if_body.generate_ir(asm_data, stack_data, global_asm_data);
                result.merge(&if_body_asm);

                //jump to the end of the if/else block
                result.add_instruction(AsmOperation::JMPCC {
                    label: if_end_label.clone(),
                    comparison: AsmComparison::ALWAYS,//unconditional jump
                });

                if let Some(else_body) = else_body {
                    //there is code in the else block

                    let (else_body_asm, _) = else_body.generate_ir(asm_data, stack_data, global_asm_data);

                    //start of the else block
                    result.add_instruction(AsmOperation::Label(else_label));//add label
                    result.merge(&else_body_asm);//generate the body of the else statement
                }

                //after if/else are complete, jump here
                result.add_instruction(AsmOperation::Label(if_end_label));

            }
        }

        (result, None)
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