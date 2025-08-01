use crate::{asm_gen_data::{AsmData, GlobalAsmData}, assembly::{assembly::Assembly, comparison::AsmComparison, operand::{immediate::ImmediateValue, register::GPRegister, Operand}, operation::{AsmOperation, Label}}, ast_metadata::ASTMetadata, block_statement::StatementOrDeclaration, compilation_state::label_generator::LabelGenerator, data_type::{base_type::BaseType, recursive_data_type::DataType}, debugging::ASTDisplay, expression::expression::{self, Expression}, expression_visitors::{data_type_visitor::GetDataTypeVisitor, put_scalar_in_acc::ScalarInAccVisitor}, lexer::{keywords::Keyword, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, number_literal::typed_value::NumberLiteral, parse_data::ParseData, stack_allocation::StackAllocator, statement::Statement};
use colored::Colorize;
use memory_size::MemorySize;
use unwrap_let::unwrap_let;

/**
 * this handles if statements and other conditionals
 */
pub enum IterationStatement{
    FOR{
        initialisation: Option<Box<StatementOrDeclaration>>,//can't be anything fancy like a scope or if statement, but expressions and declarations are OK
        condition: Expression,
        increment: Option<Expression>,

        local_scope_data: ParseData,//metadata to help with assembly generation

        body: Box<Statement>
    },
    WHILE {
        condition: Expression,
        body: Box<Statement>,
    }
}

impl IterationStatement {
    /**
     * outer_scope_data should never be modified, just 
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, outer_scope_data: &mut ParseData, struct_label_gen: &mut LabelGenerator) -> Option<ASTMetadata<IterationStatement>> {
        let mut curr_queue_idx = previous_queue_idx.clone();

        let kw = if let Some(Token::KEYWORD(x)) = tokens_queue.consume(&mut curr_queue_idx, outer_scope_data) {x} else {return None;};
        
        match kw {
            Keyword::FOR => {
                //important: clone the local variables and enums, to prevent inner definitions from leaking out to outer scopes
                let mut in_loop_data = outer_scope_data.clone_for_new_scope();
                
                let closecurly_idx = tokens_queue.find_matching_close_bracket(curr_queue_idx.index);
                assert!(Token::PUNCTUATOR(Punctuator::OPENCURLY) == tokens_queue.consume(&mut curr_queue_idx, &in_loop_data).unwrap());//ensure opening parenthesis

                let items_slice = TokenQueueSlice{
                    index: curr_queue_idx.index,
                    max_index: closecurly_idx
                };

                let items = tokens_queue.split_outside_parentheses(&items_slice, |x| *x == Token::PUNCTUATOR(Punctuator::SEMICOLON), &TokenSearchType::skip_nothing());
                assert!(items.len() == 3);
                let (init_slice, condition_slice, increment_slice) = (&items[0], &items[1], &items[2]);//get the slices that I need

                let init_with_semicolon = TokenQueueSlice {
                    index:init_slice.index,
                    max_index:init_slice.max_index+1
                };

                //TODO let for loops have blank slices: for(;true;) IS VALID

                //get initialisation command, or None
                let initialisation = StatementOrDeclaration::try_consume(tokens_queue, &init_with_semicolon, &mut in_loop_data, struct_label_gen).and_then(|ast_data| Some(Box::new(ast_data.resultant_tree)));
                //get loop condition or if none, a constant "true" value
                let condition = expression::try_consume_whole_expr(tokens_queue, &condition_slice, &mut in_loop_data, struct_label_gen)
                    .unwrap_or(Expression::NUMBERLITERAL(NumberLiteral::from(1)));
                //get increment or None
                let increment = expression::try_consume_whole_expr(tokens_queue, &increment_slice, &mut in_loop_data, struct_label_gen);

                //consume the "for (;;)" part
                curr_queue_idx = TokenQueueSlice{
                    index: closecurly_idx + 1,
                    max_index: curr_queue_idx.max_index
                };

                //consume the body
                let ASTMetadata{ remaining_slice, resultant_tree: loop_body } = Statement::try_consume(tokens_queue, &curr_queue_idx, &mut in_loop_data, struct_label_gen).unwrap();
                curr_queue_idx = remaining_slice;

                Some(ASTMetadata{
                    resultant_tree: Self::FOR { initialisation, condition, increment, body: Box::new(loop_body), local_scope_data: in_loop_data }, 
                    remaining_slice: curr_queue_idx
                })
            },
            Keyword::WHILE => {
                
                let closecurly_idx = tokens_queue.find_matching_close_bracket(curr_queue_idx.index);
                assert!(Token::PUNCTUATOR(Punctuator::OPENCURLY) == tokens_queue.consume(&mut curr_queue_idx, outer_scope_data).unwrap());//ensure opening parenthesis

                let condition_slice = TokenQueueSlice{
                    index: curr_queue_idx.index,
                    max_index: closecurly_idx
                };

                let condition = expression::try_consume_whole_expr(tokens_queue, &condition_slice, outer_scope_data, struct_label_gen).unwrap();

                //consume the "while ()" part
                curr_queue_idx = TokenQueueSlice{
                    index: closecurly_idx + 1,
                    max_index: curr_queue_idx.max_index
                };

                //consume the body
                let ASTMetadata{ remaining_slice, resultant_tree: loop_body} = Statement::try_consume(tokens_queue, &curr_queue_idx, outer_scope_data, struct_label_gen).unwrap();
                curr_queue_idx = remaining_slice;

                Some(ASTMetadata{
                    resultant_tree: Self::WHILE { condition: condition, body: Box::new(loop_body)  }, 
                    remaining_slice: curr_queue_idx, 
                })
            }
            _ => None
        }
    }

    pub fn generate_assembly(&self, asm_data: &AsmData, stack_data: &mut StackAllocator, global_asm_data: &mut GlobalAsmData) -> Assembly {
        let mut result = Assembly::make_empty();

        let generic_label = global_asm_data.label_gen_mut().generate_label();
        let loop_start_label = Label::Local(format!("{}_loop_start", generic_label));
        let loop_end_label = Label::Local(format!("{}_loop_end", generic_label));

        //overwrite asm_data, substituting the label to go to if "break;" is called
        let asm_data = &asm_data.clone_for_new_loop(loop_end_label.clone());

        match self {
            Self::FOR { initialisation, condition, increment, body, local_scope_data } => {
                //overwrite asm_data by creating new scope
                let asm_data = asm_data.clone_for_new_scope(local_scope_data, stack_data);
                
                unwrap_let!(DataType::RAW(BaseType::Scalar(condition_type)) = condition.accept(&mut GetDataTypeVisitor {asm_data: &asm_data}));

                let loop_increment_label = Label::Local(format!("{}_loop_increment", generic_label));//extra label required for for loops

                //write to stack data whilst generating assembly for initialising the loop body
                let init_asm = match initialisation {
                    Some(x) => x.generate_assembly(&asm_data, stack_data, global_asm_data),
                    None => Assembly::make_empty(),//no initialisation => blank assembly
                };
                
                //initialise the for loop anyways
                result.merge(&init_asm);

                result.add_instruction(AsmOperation::Label(loop_start_label.clone()));//label for loop's start

                let condition_asm = condition.accept(&mut ScalarInAccVisitor {asm_data: &asm_data, stack_data});

                result.merge(&condition_asm);//generate the condition

                //compare the result to 0
                result.add_instruction(AsmOperation::CMP {
                    rhs: Operand::Imm(ImmediateValue("0".to_string())),
                    data_type: condition_type
                });

                //if the result is 0, jump to the end of the loop
                result.add_instruction(AsmOperation::JMPCC {
                    label: loop_end_label.clone(),
                    comparison: AsmComparison::EQ,
                });

                //overwrite stack data whilst generating assembly for the loop body
                let body_asm = body.generate_assembly(&asm_data, stack_data, global_asm_data);
                result.merge(&body_asm);//generate the loop body

                result.add_instruction(AsmOperation::Label(loop_increment_label));//add label to jump to incrementing the loop

                if let Some(inc) = increment {//if there is an increment
                    let increment_asm = inc.accept(&mut ScalarInAccVisitor {asm_data: &asm_data, stack_data});
                    result.merge(&increment_asm);//apply the increment
                }

                //after increment, go to top of loop
                result.add_instruction(AsmOperation::JMPCC {
                    label: loop_start_label.clone(),
                    comparison: AsmComparison::ALWAYS,
                });

                result.add_instruction(AsmOperation::Label(loop_end_label));
            },

            Self::WHILE { condition, body } => {

                unwrap_let!(DataType::RAW(BaseType::Scalar(condition_type)) = condition.accept(&mut GetDataTypeVisitor {asm_data: &asm_data}));

                result.add_instruction(AsmOperation::Label(loop_start_label.clone())); // label for loop's start

                let condition_asm = condition.accept(&mut ScalarInAccVisitor { asm_data, stack_data });
                result.merge(&condition_asm); // generate the condition

                // compare the result to 0
                result.add_instruction(AsmOperation::CMP {
                    rhs: Operand::Imm(ImmediateValue("0".to_string())),
                    data_type: condition_type,
                });

                // if the result is 0, jump to the end of the loop
                result.add_instruction(AsmOperation::JMPCC {
                    label: loop_end_label.clone(),
                    comparison: AsmComparison::EQ,
                });

                // generate the loop body
                let body_asm = body.generate_assembly(asm_data, stack_data, global_asm_data);
                result.merge(&body_asm);

                // after loop complete, go to top of loop
                result.add_instruction(AsmOperation::JMPCC {
                    label: loop_start_label,
                    comparison: AsmComparison::ALWAYS,
                });

                result.add_instruction(AsmOperation::Label(loop_end_label));
            }
        }
        
        result
    }
}

impl ASTDisplay for IterationStatement {
    fn display_ast(&self, f: &mut crate::debugging::TreeDisplayInfo) {
        f.write(&"loop".red().to_string());
        f.indent();
        match self {
            IterationStatement::FOR { initialisation, condition, increment, local_scope_data:_, body } => {
                if let Some(init) = initialisation {
                    f.write(&"initialisation".red().to_string());
                    f.indent();
                    init.display_ast(f);
                    f.dedent();
                }

                f.write(&"condition".green().to_string());
                f.indent();
                condition.display_ast(f);
                f.dedent();

                if let Some(inc) = increment {
                    f.write(&"increment".red().to_string());
                    f.indent();
                    inc.display_ast(f);
                    f.dedent();
                }

                f.write(&"body".red().to_string());
                f.indent();
                body.display_ast(f);
                f.dedent();
            },
            IterationStatement::WHILE { condition, body } => {
                f.write(&"condition".green().to_string());
                f.indent();
                condition.display_ast(f);
                f.dedent();

                f.write(&"body".red().to_string());
                f.indent();
                body.display_ast(f);
                f.dedent();
            },
        }
        f.dedent();
    }
}