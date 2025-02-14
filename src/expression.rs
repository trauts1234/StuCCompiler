use crate::{asm_boilerplate, ast_metadata::ASTMetadata, lexer::{token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue, punctuator::{MathematicalOperator, Punctuator}}, memory_size::MemoryLayout, number_literal::NumberLiteral, stack_variables::StackVariables};
use std::fmt::Write;

#[derive(Debug, Clone)]
pub enum Expression {
    STACKVAR(MemoryLayout),//offset from bp
    NUMBER(NumberLiteral),
    BINARYEXPR(Box<Expression>, MathematicalOperator, Box<Expression>)
}

impl Expression {
    /**
     * tries to consume an expression, terminated by a semicolon, and returns None if this is not possible
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, local_variables: &StackVariables) -> Option<ASTMetadata<Expression>> {
        let semicolon_idx = tokens_queue.find_closure_in_slice(&previous_queue_idx, false, |x| *x == Token::PUNCTUATOR(Punctuator::SEMICOLON))?;
        //define the slice that we are going to try and parse
        let attempt_slice = TokenQueueSlice {
            index: previous_queue_idx.index,
            max_index: semicolon_idx.index
        };

        //warning! the line of code:
        //1+1;
        //will cause a memory leak on the stack?!
        //as 1, 1 are pushed, popped, added, then 2 is pushed but nothing pops it
        //is this a problem? probably not, but it is a memory leak

        match Expression::try_consume_whole_expr(tokens_queue, &attempt_slice, local_variables) {
            Some(expr) => {
                Some(ASTMetadata{resultant_tree: expr, remaining_slice: semicolon_idx.next_clone(), extra_stack_used: MemoryLayout::new()})
            },
            None => None
        }
    }
    /**
     * tries to parse the tokens queue starting at previous_queue_idx, to find an expression
     * returns an expression(entirely consumed), else none
     */
    pub fn try_consume_whole_expr(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, local_variables: &StackVariables) -> Option<Expression> {
        let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        match curr_queue_idx.get_slice_size() {
            0 => panic!("not expecting this, maybe it is not an expression"),

            1 => {
                //1 token left, check if it is a number
                match tokens_queue.peek(& curr_queue_idx)? {
                    Token::NUMBER(num) => {
                        tokens_queue.consume(&mut curr_queue_idx);
                        Some(Expression::NUMBER(num))
                    },
                    Token::IDENTIFIER(var_name) => {
                        Some(Expression::STACKVAR(local_variables.get_variable_bp_offset(&var_name).unwrap()))
                    },
                    _ => None
                }
            },

            _ => {
                //TODO handle brackets outside of operator

                for precedence_required in MathematicalOperator::min_precedence()..=MathematicalOperator::max_precedence() {
                    //try to find an operator, starting with the operators that bind the hardest (small precedence)

                    //find which direction the operators should be considered
                    let associative_direction = MathematicalOperator::get_associativity_direction(precedence_required);

                    //make a closure that detects operators that match what we want
                    let operator_matching_closure = |x: &Token| {
                        x.as_punctuator()//get punctuator if it can
                        .and_then(|punc| punc.as_mathematical_operator())//get operator from the punctuator if it can
                        .and_then(|op| Some(op.get_precedence_level()))//get precedence level
                        .is_some_and(|precedence| precedence == precedence_required)//ensure that it is the correct precedence level
                    };

                    let operator_indexes = tokens_queue.find_closure_matches(&curr_queue_idx, associative_direction, operator_matching_closure);

                    for operator_idx in operator_indexes {
                        //try to find an operator
                        //note that the operator_idx is a slice of just the operator

                        match try_parse_binary_expr(tokens_queue, &curr_queue_idx, &operator_idx, local_variables) {
                            Some(x) => {return Some(x);}
                            None => {
                                println!("failed to match at precedence {}, even though I found an operator at index {}", precedence_required, operator_idx.index);
                                continue;
                            }
                        }

                    }
                }

                None//tried everything and still failed
            }
        }
    }

    /**
     * puts the result of the expression on top of the stack
     */
    pub fn generate_assembly(&self) -> String {
        let mut result = String::new();

        match self {
            Expression::STACKVAR(stack_offset) => {
                writeln!(result, "mov rax, [rbp-{}]", stack_offset.size_bytes()).unwrap();//load the value from memory
                writeln!(result, "push rax").unwrap();//push the value on to the stack
            },
            Expression::NUMBER(number_literal) => {
                writeln!(result, "mov rax, {}", number_literal.nasm_format()).unwrap();
                writeln!(result, "push rax").unwrap();
            },
            Expression::BINARYEXPR(lhs, operator, rhs) => {
                match operator {
                    MathematicalOperator::ADD => {
                        //put values on stack
                        write!(result, "{}", lhs.generate_assembly()).unwrap();
                        write!(result, "{}", rhs.generate_assembly()).unwrap();

                        writeln!(result, "{}", asm_boilerplate::I32_ADD).unwrap();
                        
                    },
                    MathematicalOperator::SUBTRACT => {
                        //put values on stack
                        write!(result, "{}", lhs.generate_assembly()).unwrap();
                        write!(result, "{}", rhs.generate_assembly()).unwrap();

                        writeln!(result, "{}", asm_boilerplate::I32_SUBTRACT).unwrap();
                    }
                    MathematicalOperator::MULTIPLY => {
                        //put values on stack
                        write!(result, "{}", lhs.generate_assembly()).unwrap();
                        write!(result, "{}", rhs.generate_assembly()).unwrap();

                        writeln!(result, "{}", asm_boilerplate::I32_MULTIPLY).unwrap();
                    },
                    MathematicalOperator::ASSIGN => {
                        //put address of lvalue on stack
                        write!(result, "{}", lhs.put_lvalue_addr_on_stack()).unwrap();
                        //put the value to assign on stack
                        write!(result, "{}", rhs.generate_assembly()).unwrap();
                        //pop the value to assign
                        writeln!(result, "pop rax").unwrap();
                        //pop address to assign to
                        writeln!(result, "pop rbx").unwrap();
                        //save to memory
                        writeln!(result, "mov [rbx], rax").unwrap();
                    },
                    MathematicalOperator::DIVIDE => {
                        //put values on stack
                        write!(result, "{}", lhs.generate_assembly()).unwrap();
                        write!(result, "{}", rhs.generate_assembly()).unwrap();

                        writeln!(result, "{}", asm_boilerplate::I32_DIVIDE).unwrap();
                    }
                }
            },
        };
        result
    }

    fn put_lvalue_addr_on_stack(&self) -> String {
        let mut result = String::new();

        match self {
            Expression::STACKVAR(stack_offset) => {
                writeln!(result, "lea rax, [rbp-{}]", stack_offset.size_bytes()).unwrap();//calculate the address of the variable
                writeln!(result, "push rax").unwrap();//push the address on to the stack
            },
            _ => panic!("tried to generate assembly to assign to a non-lvalue")
        };
        result
    }
}

/**
 * tries to parse the left and right hand side of operator_idx.index, as a binary expression e.g 1 + 2 split by "+"
 * if this parse was successful, an expression is returned
 * else, you get None
 */
fn try_parse_binary_expr(tokens_queue: &mut TokenQueue, curr_queue_idx: &TokenQueueSlice, operator_idx: &TokenQueueSlice, local_variables: &StackVariables) -> Option<Expression> {
    //split to before and after the operator
    let (left_part, right_part) = tokens_queue.split_to_slices(operator_idx, curr_queue_idx);

    assert!(right_part.get_slice_size() != 2);

    //try and parse the left and right hand sides, propogating errors
    let parsed_left = Expression::try_consume_whole_expr(tokens_queue, &left_part, local_variables)?;
    let parsed_right = Expression::try_consume_whole_expr(tokens_queue, &right_part, local_variables)?;

    let operator = tokens_queue.peek(&operator_idx).expect("couldn't peek")
        .as_punctuator().expect("couldn't cast to punctuator")
        .as_mathematical_operator().expect("couldn't cast to operator");

    Some(Expression::BINARYEXPR(Box::new(parsed_left), operator, Box::new(parsed_right)))
}