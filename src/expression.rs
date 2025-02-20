use crate::{asm_boilerplate, asm_generation::{self, Register}, ast_metadata::ASTMetadata, compilation_state::{functions::FunctionList, stack_variables::StackVariables}, declaration::AddressedDeclaration, function_call::FunctionCall, lexer::{precedence, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, memory_size::MemoryLayout, number_literal::NumberLiteral, type_info::{DataType, DeclModifier}};
use std::fmt::Write;
use crate::asm_generation::{asm_line, asm_comment};

#[derive(Debug, Clone)]
pub enum Expression {
    STACKVAR(AddressedDeclaration),
    NUMBER(NumberLiteral),
    BINARYEXPR(Box<Expression>, Punctuator, Box<Expression>),
    PREFIXEXPR(Punctuator, Box<Expression>),
    FUNCCALL(FunctionCall)
}

impl Expression {
    /**
     * tries to consume an expression, terminated by a semicolon, and returns None if this is not possible
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, local_variables: &StackVariables, accessible_funcs: &FunctionList) -> Option<ASTMetadata<Expression>> {
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

        match Expression::try_consume_whole_expr(tokens_queue, &attempt_slice, local_variables, accessible_funcs) {
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
    pub fn try_consume_whole_expr(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, local_variables: &StackVariables, accessible_funcs: &FunctionList) -> Option<Expression> {
        let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        if tokens_queue.slice_is_parenthesis(&curr_queue_idx) {
            //we are an expression surrounded by brackets
            //remove the outer brackets and continue
            curr_queue_idx = TokenQueueSlice {
                index: curr_queue_idx.index+1,
                max_index: curr_queue_idx.max_index-1
            };
        }

        match curr_queue_idx.get_slice_size() {
            0 => None,//panic!("not expecting this, maybe it is not an expression"),

            1 => {
                //1 token left, check if it is a number
                match tokens_queue.peek(& curr_queue_idx)? {
                    Token::NUMBER(num) => {
                        tokens_queue.consume(&mut curr_queue_idx);
                        Some(Expression::NUMBER(num))
                    },
                    Token::IDENTIFIER(var_name) => {
                        Some(Expression::STACKVAR(local_variables.get_variable(&var_name).unwrap()))
                    },
                    _ => None
                }
            },

            _ => {

                for precedence_required in (precedence::min_precedence()..=precedence::max_precedence()).rev() {
                    //try to find an operator to split the expression by, starting with the operators that bind the weakest (high precedence)

                    //find which direction the operators should be considered
                    //true is l->r, which means that if true, scan direction for splitting points should be reversed
                    let associative_direction = precedence::get_associativity_direction(precedence_required);

                    if associative_direction {
                        //look for unary postfix
                        assert!(curr_queue_idx.max_index <= tokens_queue.tokens.len());

                        if precedence_required == 1 {
                            match try_parse_array_index(tokens_queue, &curr_queue_idx, local_variables, accessible_funcs) {
                                Some(x) => {return Some(x)},
                                None => {}
                            }

                            if let Some(func) = FunctionCall::try_consume_whole_expr(tokens_queue, &curr_queue_idx, local_variables, accessible_funcs) {
                                return Some(Expression::FUNCCALL(func));
                            }
                        }

                    } else {
                        //look for unary prefix as association is right to left
                        let first_token = tokens_queue.peek(&curr_queue_idx)?;

                        let starts_with_valid_prefix = first_token
                            .as_punctuator()
                            .and_then(|punc| punc.as_unary_prefix_precedence())
                            .is_some_and(|precedence| precedence == precedence_required);

                        if starts_with_valid_prefix {
                            match try_parse_unary_prefix(tokens_queue, &curr_queue_idx, local_variables, accessible_funcs) {
                                Some(x) => {return Some(x);},
                                None => {
                                    println!("failed to parse unary prefix")
                                }
                            }
                        }
                    }

                    //make a closure that detects operators that match what we want
                    let operator_matching_closure = |x: &Token| {
                        x.as_punctuator()//get punctuator
                        .and_then(|punc| punc.as_binary_operator_precedence())//try and get the punctuator as a binary operator's precedence
                        .is_some_and(|precedence| precedence == precedence_required)//ensure that it is the correct precedence level
                    };

                    //when searching, avoid splitting by something found inside brackets
                    let exclusions = TokenSearchType{
                        skip_in_curly_brackets: true,
                        skip_in_square_brackets: true,
                    };

                    let operator_indexes = tokens_queue.find_closure_matches(&curr_queue_idx, associative_direction, operator_matching_closure, &exclusions);

                    for operator_idx in operator_indexes {
                        //try to find an operator
                        //note that the operator_idx is a slice of just the operator

                        match try_parse_binary_expr(tokens_queue, &curr_queue_idx, &operator_idx, local_variables, accessible_funcs) {
                            Some(x) => {return Some(x);}
                            None => {
                                continue;
                            }
                        }

                    }
                }

                None//tried everything and still failed
            }
        }
    }

    pub fn get_data_type(&self) -> DataType {
        match self {
            Expression::STACKVAR(decl) => decl.decl.data_type.clone(),
            Expression::NUMBER(num_literal) => num_literal.get_data_type().data_type,
            Expression::PREFIXEXPR(prefix, rhs) => {
                match prefix {
                    Punctuator::AMPERSAND => DataType {
                        type_info: rhs.get_data_type().type_info,//same base type as rhs
                        modifiers: std::iter::once(DeclModifier::POINTER).chain(rhs.get_data_type().modifiers.clone()).collect(), //pointer to whatever rhs is
                    },
                    Punctuator::ASTERISK => DataType {
                        type_info: rhs.get_data_type().type_info,
                        modifiers: rhs.get_data_type().modifiers[1..].to_vec(),//remove the pointer info, as it has been dereferenced
                    },
                    _ => panic!("tried getting data type of a not-implemented prefix")
                }
            },
            Expression::BINARYEXPR(lhs, op, rhs) => {
                match op {
                    Punctuator::PLUS => DataType::calculate_promoted_type_arithmetic(&lhs.get_data_type(), &rhs.get_data_type()),
                    Punctuator::DASH => DataType::calculate_promoted_type_arithmetic(&lhs.get_data_type(), &rhs.get_data_type()),

                    Punctuator::ASTERISK => DataType::calculate_promoted_type_arithmetic(&lhs.get_data_type(), &rhs.get_data_type()),
                    Punctuator::FORWARDSLASH => DataType::calculate_promoted_type_arithmetic(&lhs.get_data_type(), &rhs.get_data_type()),

                    Punctuator::EQUALS => lhs.get_data_type(),//assigning, rhs must be converted to lhs

                    _ => panic!("data type calculation for this binary operator is not implemented")
                }
            },
            Expression::FUNCCALL(func_data) => {
                func_data.get_data_type()
            }

        }
    }

    /**
     * puts the result of the expression on top of the stack
     */
    pub fn generate_assembly(&self) -> String {
        let mut result = String::new();
        let promoted_type = self.get_data_type();

        let acc_reg = asm_generation::generate_reg_name(&promoted_type.memory_size(), Register::ACC);//accumulator
        let secondary_reg = asm_generation::generate_reg_name(&promoted_type.memory_size(), Register::SECONDARY);

        match self {
            Expression::STACKVAR(decl) => {

                if decl.decl.data_type.is_array() {
                    let ptr_reg = asm_generation::generate_reg_name(&MemoryLayout::from_bytes(8), Register::ACC);
                    //getting an array, decays to a pointer
                    asm_comment!(result, "decaying array {} to pointer", decl.decl.name);
                    asm_line!(result, "lea {}, [rbp-{}]", ptr_reg, decl.stack_offset.size_bytes());
                    asm_line!(result, "{}", asm_boilerplate::push_reg(&ptr_reg));
                } else {
                    let reg_name = asm_generation::generate_reg_name(&decl.decl.data_type.memory_size(), Register::ACC);//decide which register size is appropriate for this variable
                    asm_comment!(result, "reading variable: {} to register {}", decl.decl.name, reg_name);

                    asm_line!(result, "mov {}, [rbp-{}]", reg_name, decl.stack_offset.size_bytes());//get the value from its address on the stack
                    asm_line!(result, "{}", asm_boilerplate::push_reg(&reg_name));//push on to top of stack
                }
            },
            Expression::NUMBER(number_literal) => {
                let reg_name = asm_generation::generate_reg_name(&number_literal.get_data_type().data_type.memory_size(), Register::ACC);//decide which register should be used temporarily to push the value
                asm_comment!(result, "reading number literal: {} via register {}", number_literal.nasm_format(), reg_name);

                asm_line!(result, "mov {}, {}", reg_name, number_literal.nasm_format());
                asm_line!(result, "{}", asm_boilerplate::push_reg(&reg_name));
            },
            Expression::PREFIXEXPR(operator, rhs) => {
                match operator {
                    Punctuator::AMPERSAND => {
                        asm_comment!(result, "getting address of something");
                        //put address of the right hand side on the stack
                        asm_line!(result, "{}", rhs.put_lvalue_addr_on_stack());
                    },
                    Punctuator::ASTERISK => {
                        asm_comment!(result, "dereferencing pointer");
                        // put the _pointer's_ memory location on the stack
                        asm_line!(result, "{}", rhs.generate_assembly());
                        asm_line!(result, "{}", asm_boilerplate::pop_reg("rax"));//load the pointer into RAX

                        assert!(rhs.get_data_type().memory_size().size_bits() == 64);//must be a 64 bit address
                        
                        asm_line!(result, "mov rax, [rax]");
                        asm_line!(result, "{}", asm_boilerplate::push_reg("rax"));
                    },
                    _ => panic!("operator to unary prefix is invalid")
                }
            }
            Expression::BINARYEXPR(lhs, operator, rhs) => {
                match operator {
                    Punctuator::PLUS => {
                        asm_comment!(result, "adding numbers using {}", acc_reg);

                        asm_line!(result, "{}", lhs.generate_assembly());//put lhs on stack
                        asm_line!(result, "{}", asm_boilerplate::cast_from_stack(&lhs.get_data_type(), &promoted_type));//cast to the correct type

                        if rhs.get_data_type().is_pointer() {//adding pointer to int
                            //you can only add pointer and number here, as per the C standard

                            //get the size of rhs when it is dereferenced
                            let rhs_dereferenced_size_bytes = rhs.get_data_type().remove_outer_modifier().memory_size().size_bytes();
                            //convert this number to a string
                            let rhs_deref_size_str = NumberLiteral::try_new(&rhs_dereferenced_size_bytes.to_string()).unwrap().nasm_format();

                            asm_comment!(result, "rhs is a pointer. make lhs {} times bigger", rhs_deref_size_str);

                            assert!(promoted_type.memory_size().size_bytes() == 8);
                            asm_line!(result, "{}", asm_boilerplate::pop_reg("rax"));//pop the value of lhs(certainly promoted to 64 bit pointer)
                            asm_line!(result, "mov rcx, {}", rhs_deref_size_str);//get the size of value pointed to by rhs
                            asm_line!(result, "mul rcx");//multiply lhs by the size of value pointed to by rhs, so that +1 would skip along 1 value, not 1 byte
                            asm_line!(result, "{}", asm_boilerplate::push_reg("rax"));//save the result back to stack
                        }

                        asm_line!(result, "{}", rhs.generate_assembly());
                        asm_line!(result, "{}", asm_boilerplate::cast_from_stack(&rhs.get_data_type(), &promoted_type));

                        if lhs.get_data_type().is_pointer() {
                            //you can only add pointer and number here, as per the C standard
                            //get the size of lhs when it is dereferenced
                            let lhs_dereferenced_size_bytes = lhs.get_data_type().remove_outer_modifier().memory_size().size_bytes();
                            //convert this number to a string
                            let lhs_deref_size_str = NumberLiteral::try_new(&lhs_dereferenced_size_bytes.to_string()).unwrap().nasm_format();

                            asm_comment!(result, "lhs is a pointer. make rhs {} times bigger", lhs_deref_size_str);

                            assert!(promoted_type.memory_size().size_bytes() == 8);
                            asm_line!(result, "{}", asm_boilerplate::pop_reg("rax"));//pop the value of rhs(certainly promoted to 64 bit pointer)
                            asm_line!(result, "mov rcx, {}", lhs_deref_size_str);//get the size of value pointed to by rhs
                            asm_line!(result, "mul rcx");//multiply rhs by the size of value pointed to by lhs, so that +1 would skip along 1 value, not 1 byte
                            asm_line!(result, "{}", asm_boilerplate::push_reg("rax"));//save the result back to stack
                        }

                        asm_line!(result, "{}\n{}", asm_boilerplate::pop_reg(&secondary_reg), asm_boilerplate::pop_reg(&acc_reg));

                        asm_line!(result, "add {}, {}", acc_reg, secondary_reg);

                        asm_line!(result, "{}", asm_boilerplate::push_reg(&acc_reg));
                        
                    },
                    Punctuator::DASH => {
                        asm_comment!(result, "subtracting numbers");
                        //put values on stack
                        asm_line!(result, "{}", lhs.generate_assembly());
                        asm_line!(result, "{}", asm_boilerplate::cast_from_stack(&lhs.get_data_type(), &promoted_type));

                        asm_line!(result, "{}", rhs.generate_assembly());
                        asm_line!(result, "{}", asm_boilerplate::cast_from_stack(&rhs.get_data_type(), &promoted_type));

                        asm_line!(result, "{}\n{}", asm_boilerplate::pop_reg(&secondary_reg), asm_boilerplate::pop_reg(&acc_reg));

                        asm_line!(result, "sub {}, {}", acc_reg, secondary_reg);

                        asm_line!(result, "{}", asm_boilerplate::push_reg(&acc_reg));
                    }
                    Punctuator::ASTERISK => {
                        asm_comment!(result, "multiplying numbers");
                        //put values on stack
                        asm_line!(result, "{}", lhs.generate_assembly());
                        asm_line!(result, "{}", asm_boilerplate::cast_from_stack(&lhs.get_data_type(), &promoted_type));

                        asm_line!(result, "{}", rhs.generate_assembly());
                        asm_line!(result, "{}", asm_boilerplate::cast_from_stack(&rhs.get_data_type(), &promoted_type));

                        asm_line!(result, "{}\n{}", asm_boilerplate::pop_reg(&secondary_reg), asm_boilerplate::pop_reg(&acc_reg));

                        assert!(!promoted_type.underlying_type_is_unsigned() || promoted_type.is_pointer());//unsigned multiply??

                        asm_line!(result, "imul {}, {}", acc_reg, secondary_reg);

                        asm_line!(result, "{}", asm_boilerplate::push_reg(&acc_reg));
                    },
                    Punctuator::EQUALS => {//assign
                        let rhs_reg_name = asm_generation::generate_reg_name(&promoted_type.memory_size(), Register::ACC);//accumulator for the first item
                        //put address of lvalue on stack
                        asm_line!(result, "{}", lhs.put_lvalue_addr_on_stack());
                        //put the value to assign on stack
                        asm_line!(result, "{}", rhs.generate_assembly());
                        //cast to the same type as lhs
                        asm_line!(result, "{}", asm_boilerplate::cast_from_stack(&rhs.get_data_type(), &promoted_type));

                        asm_comment!(result, "assigning to a stack variable");

                        //pop the value to assign
                        asm_line!(result, "{}", asm_boilerplate::pop_reg(&rhs_reg_name));
                        //pop address to assign to
                        asm_line!(result, "{}", asm_boilerplate::pop_reg("rcx"));
                        //save to memory
                        asm_line!(result, "mov [rcx], {}", rhs_reg_name);
                    },
                    Punctuator::FORWARDSLASH => {
                        asm_comment!(result, "dividing numbers");
                        //put values on stack
                        asm_line!(result, "{}", lhs.generate_assembly());
                        asm_line!(result, "{}", asm_boilerplate::cast_from_stack(&lhs.get_data_type(), &promoted_type));

                        asm_line!(result, "{}", rhs.generate_assembly());
                        asm_line!(result, "{}", asm_boilerplate::cast_from_stack(&rhs.get_data_type(), &promoted_type));

                        asm_line!(result, "{}\n{}", asm_boilerplate::pop_reg(&secondary_reg), asm_boilerplate::pop_reg(&acc_reg));

                        match (promoted_type.memory_size().size_bytes(), promoted_type.underlying_type_is_unsigned()) {
                            (4,false) => {
                                asm_line!(result, "{}", asm_boilerplate::I32_DIVIDE_AX_BY_CX);
                            },
                            (8,false) => {
                                asm_line!(result, "{}", asm_boilerplate::I64_DIVIDE_AX_BY_CX);
                            }
                            _ => panic!("unsupported operands")
                        }

                        asm_line!(result, "{}", asm_boilerplate::push_reg(&acc_reg));
                    },
                    _ => panic!("operator to binary expression is invalid")
                }
            },
            Expression::FUNCCALL(call_data) => {
                asm_line!(result, "{}", call_data.generate_assembly());
            }
        };
        result
    }

    fn put_lvalue_addr_on_stack(&self) -> String {
        let mut result = String::new();

        match self {
            Expression::STACKVAR(decl) => {
                asm_comment!(result, "getting address of variable: {}", decl.decl.name);
                asm_line!(result, "lea rax, [rbp-{}]", decl.stack_offset.size_bytes());//calculate the address of the variable
                asm_line!(result, "{}", asm_boilerplate::push_reg("rax"));//push the address on to the stack
            },
            Expression::PREFIXEXPR(Punctuator::ASTERISK, expr_box) => {
                //&*x == x
                asm_comment!(result, "getting address of a dereference");
                asm_line!(result, "{}", &expr_box.generate_assembly());
            }
            _ => panic!("tried to generate assembly to assign to a non-lvalue")
        };
        result
    }
}

/**
 * tries to parse the expression as a unary prefix and the operand, for example ++x or *(x->foo)
 * if the parse was successful, an expression is returned
 * else, you get None
 */
fn try_parse_unary_prefix(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, local_variables: &StackVariables, accessible_funcs: &FunctionList) -> Option<Expression> {
    let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);
    
    let punctuator = tokens_queue.consume(&mut curr_queue_idx)?.as_punctuator()?;//get punctuator

    punctuator.as_unary_prefix_precedence()?;//ensure the punctuator is a valid unary prefix

    let operand = Expression::try_consume_whole_expr(tokens_queue, &curr_queue_idx, local_variables, accessible_funcs)?;

    Some(Expression::PREFIXEXPR(punctuator, Box::new(operand)))
}

/**
 * tries to parse the left and right hand side of operator_idx.index, as a binary expression e.g 1 + 2 split by "+"
 * if this parse was successful, an expression is returned
 * else, you get None
 */
fn try_parse_binary_expr(tokens_queue: &mut TokenQueue, curr_queue_idx: &TokenQueueSlice, operator_idx: &TokenQueueSlice, local_variables: &StackVariables, accessible_funcs: &FunctionList) -> Option<Expression> {
    //split to before and after the operator
    let (left_part, right_part) = tokens_queue.split_to_slices(operator_idx, curr_queue_idx);

    //try and parse the left and right hand sides, propogating errors
    let parsed_left = Expression::try_consume_whole_expr(tokens_queue, &left_part, local_variables, accessible_funcs)?;
    let parsed_right = Expression::try_consume_whole_expr(tokens_queue, &right_part, local_variables, accessible_funcs)?;

    let operator = tokens_queue.peek(&operator_idx).expect("couldn't peek")
        .as_punctuator().expect("couldn't cast to punctuator");

    Some(Expression::BINARYEXPR(Box::new(parsed_left), operator, Box::new(parsed_right)))
}

fn try_parse_array_index(tokens_queue: &mut TokenQueue, curr_queue_idx: &TokenQueueSlice, local_variables: &StackVariables, accessible_funcs: &FunctionList) -> Option<Expression> {
    //look for unary postfixes as association is left to right
    let last_token = tokens_queue.peek_back(&curr_queue_idx)?;

    //handle array indexing as that is a special case of binary operator
    if last_token == Token::PUNCTUATOR(Punctuator::CLOSESQUARE) {
        let square_open_idx = tokens_queue.find_matching_open_bracket(curr_queue_idx.max_index-1);//-1 as max index is exclusive

        let index_slice = TokenQueueSlice {//index to array, pointer etc
            index: square_open_idx+1,
            max_index: curr_queue_idx.max_index-1
        };
        let array_slice = TokenQueueSlice {//array or pointer etc.
            index: curr_queue_idx.index,
            max_index: square_open_idx
        };

        let index_expr = Expression::try_consume_whole_expr(tokens_queue, &index_slice, local_variables, accessible_funcs)?;
        let array_expr = Expression::try_consume_whole_expr(tokens_queue, &array_slice, local_variables, accessible_funcs)?;

        //a[b] == *(a+b) in C
        return Some(
            Expression::PREFIXEXPR(Punctuator::ASTERISK, Box::new(//dereference
                Expression::BINARYEXPR(Box::new(array_expr), Punctuator::PLUS, Box::new(index_expr))//pointer plus index
            ))
        )
    }

    None
}