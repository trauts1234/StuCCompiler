use crate::{asm_boilerplate::{self, mov_reg}, asm_generation::{LogicalRegister, PhysicalRegister, RegisterName}, ast_metadata::ASTMetadata, compilation_state::{functions::FunctionList, stack_variables::StackVariables}, data_type::{base_type::BaseType, data_type::DataType, type_modifier::DeclModifier}, declaration::AddressedDeclaration, function_call::FunctionCall, lexer::{precedence, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, memory_size::MemoryLayout, number_literal::NumberLiteral, string_literal::StringLiteral};
use std::fmt::Write;
use crate::asm_generation::{asm_line, asm_comment};

const PTR_SIZE: MemoryLayout = MemoryLayout::from_bytes(8);

#[derive(Debug, Clone)]
pub enum Expression {
    STACKVAR(AddressedDeclaration),
    NUMBER(NumberLiteral),
    STRINGLIT(StringLiteral),
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

        println!("{:?}", tokens_queue.get_slice(&curr_queue_idx));

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
                    Token::STRING(string_lit) => {
                        Some(Expression::STRINGLIT(string_lit))
                    }
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
            Expression::STRINGLIT(str_literal) => str_literal.get_data_type(),
            Expression::PREFIXEXPR(prefix, rhs) => {
                match prefix {
                    Punctuator::AMPERSAND => {
                        let mut pointer_modifiers = rhs.get_data_type().get_modifiers().to_vec();
                        pointer_modifiers.insert(0, DeclModifier::POINTER);//pointer to whatever rhs is

                        DataType::new_from_base_type(rhs.get_data_type().underlying_type(), &pointer_modifiers)
                    },
                    Punctuator::ASTERISK => DataType::new_from_base_type(
                        rhs.get_data_type().underlying_type(), 
                        &rhs.get_data_type().get_modifiers()[1..].to_vec()//remove initial "pointer to x" from modifiers
                    ),
                    Punctuator::DASH | Punctuator::PLUSPLUS => DataType::calculate_unary_type_arithmetic(&rhs.get_data_type()),//-x may promote x to a bigger type
                    _ => panic!("tried getting data type of a not-implemented prefix")
                }
            },
            Expression::BINARYEXPR(lhs, op, rhs) => {
                match op {
                    Punctuator::PLUS |
                    Punctuator::DASH |
                    Punctuator::ASTERISK | 
                    Punctuator::FORWARDSLASH | 
                    Punctuator::PERCENT => DataType::calculate_promoted_type_arithmetic(&lhs.get_data_type(), &rhs.get_data_type()),

                    Punctuator::EQUALS => lhs.get_data_type(),//assigning, rhs must be converted to lhs

                    Punctuator::ANGLELEFT |
                    Punctuator::ANGLERIGHT |
                    Punctuator::GREATEREQUAL |
                    Punctuator::LESSEQAUAL |
                    Punctuator::DOUBLEEQUALS => DataType::new_from_base_type(&BaseType::_BOOL, &Vec::new()),

                    _ => panic!("data type calculation for this binary operator is not implemented")
                }
            },
            Expression::FUNCCALL(func_data) => {
                func_data.get_data_type()
            }

        }
    }

    /**
     * puts the result of the expression in the accumulator
     */
    pub fn generate_assembly(&self) -> String {

        let mut result = String::new();

        match self {
            Expression::STACKVAR(decl) => {

                if decl.decl.data_type.is_array() {
                    //getting an array, decays to a pointer
                    asm_comment!(result, "decaying array {} to pointer", decl.decl.name);
                    asm_line!(result, "lea {}, [rbp-{}]", LogicalRegister::ACC.generate_reg_name(&PTR_SIZE), decl.stack_offset.size_bytes());
                } else {
                    let reg_size = &decl.decl.data_type.memory_size();//decide which register size is appropriate for this variable
                    asm_comment!(result, "reading variable: {} to register {}", decl.decl.name, LogicalRegister::ACC.generate_reg_name(reg_size));

                    asm_line!(result, "mov {}, [rbp-{}]", LogicalRegister::ACC.generate_reg_name(reg_size), decl.stack_offset.size_bytes());//get the value from its address on the stack
                }
            },
            Expression::NUMBER(number_literal) => {
                let reg_size = &number_literal.get_data_type().data_type.memory_size();//decide how much storage is needed to temporarily store the constant
                asm_comment!(result, "reading number literal: {} via register {}", number_literal.nasm_format(), LogicalRegister::ACC.generate_reg_name(reg_size));

                asm_line!(result, "mov {}, {}", LogicalRegister::ACC.generate_reg_name(reg_size), number_literal.nasm_format());
            },
            Expression::PREFIXEXPR(operator, rhs) => {
                match operator {
                    Punctuator::AMPERSAND => {
                        asm_comment!(result, "getting address of something");
                        //put address of the right hand side in acc
                        asm_line!(result, "{}", rhs.put_lvalue_addr_in_acc());
                    },
                    Punctuator::ASTERISK => {
                        asm_comment!(result, "dereferencing pointer");
                        // put the address pointed to in rax
                        asm_line!(result, "{}", rhs.generate_assembly());
                        
                        asm_line!(result, "mov rax, [rax]");//dereference pointer
                    },
                    Punctuator::DASH => {
                        asm_comment!(result, "negating something");

                        let promoted_type = self.get_data_type();

                        asm_line!(result, "{}", rhs.generate_assembly());
                        asm_line!(result, "{}", asm_boilerplate::cast_from_acc(&rhs.get_data_type(), &promoted_type));//cast to the correct type

                        asm_line!(result, "neg {}", LogicalRegister::ACC.generate_reg_name(&promoted_type.memory_size()));//negate the promoted value
                    },
                    Punctuator::PLUSPLUS => {

                        let promoted_type = self.get_data_type();
                        let rhs_type = rhs.get_data_type();

                        if let Expression::STACKVAR(var) = rhs.as_ref() {
                            asm_comment!(result, "incrementing variable {}", var.decl.get_name());
                        } else {
                            panic!("tried to prefix increment a non-variable");
                        }

                        //push &rhs
                        asm_line!(result, "{}", rhs.put_lvalue_addr_in_acc());
                        asm_line!(result, "{}", asm_boilerplate::push_reg(&PTR_SIZE, &LogicalRegister::ACC));

                        //put rhs in acc
                        asm_line!(result, "{}", rhs.generate_assembly());

                        let rhs_reg = LogicalRegister::ACC.generate_reg_name(&rhs_type.memory_size());

                        //increment rhs (in acc)
                        asm_line!(result, "inc {}", rhs_reg);

                        //pop &rhs to RCX
                        asm_line!(result, "{}", asm_boilerplate::pop_reg(&PTR_SIZE, &LogicalRegister::SECONDARY));

                        //save the new value of rhs
                        asm_line!(result, "mov [{}], {}", LogicalRegister::SECONDARY.generate_reg_name(&PTR_SIZE), LogicalRegister::ACC.generate_reg_name(&rhs_type.memory_size()));

                        asm_line!(result, "{}", asm_boilerplate::cast_from_acc(&rhs.get_data_type(), &promoted_type));//cast to the correct type
                    }
                    _ => panic!("operator to unary prefix is invalid")
                }
            }
            Expression::BINARYEXPR(lhs, operator, rhs) => {
                let promoted_type = match operator {//I already have a function for this?
                    Punctuator::EQUALS => lhs.get_data_type(),//assignment is just the lhs data size
                    _ => DataType::calculate_promoted_type_arithmetic(&lhs.get_data_type(), &rhs.get_data_type())//else find a common meeting ground
                };
                let promoted_size = &promoted_type.memory_size();

                match operator {
                    Punctuator::PLUS => {
                        asm_comment!(result, "adding {}-bit numbers", promoted_size.size_bits());

                        asm_line!(result, "{}", lhs.generate_assembly());//put lhs in acc
                        asm_line!(result, "{}", asm_boilerplate::cast_from_acc(&lhs.get_data_type(), &promoted_type));//cast to the correct type

                        if rhs.get_data_type().is_pointer() {//adding pointer to int
                            //you can only add pointer and number here, as per the C standard

                            //get the size of rhs when it is dereferenced
                            let rhs_dereferenced_size_bytes = rhs.get_data_type().remove_outer_modifier().memory_size().size_bytes();
                            //convert this number to a string
                            let rhs_deref_size_str = NumberLiteral::try_new(&rhs_dereferenced_size_bytes.to_string()).unwrap().nasm_format();
                            asm_comment!(result, "rhs is a pointer. make lhs {} times bigger", rhs_deref_size_str);

                            assert!(promoted_type.memory_size().size_bytes() == 8);
                            asm_line!(result, "mov rcx, {}", rhs_deref_size_str);//get the size of value pointed to by rhs
                            asm_line!(result, "mul rcx");//multiply lhs by the size of value pointed to by rhs, so that +1 would skip along 1 value, not 1 byte
                            
                            //lhs is now in AX
                        }

                        //save lhs to stack, as preprocessing for it is done
                        asm_line!(result, "{}", asm_boilerplate::push_reg(promoted_size, &LogicalRegister::ACC));

                        asm_line!(result, "{}", rhs.generate_assembly());//put rhs in acc
                        asm_line!(result, "{}", asm_boilerplate::cast_from_acc(&rhs.get_data_type(), &promoted_type));//cast to correct type

                        if lhs.get_data_type().is_pointer() {
                            //you can only add pointer and number here, as per the C standard
                            //get the size of lhs when it is dereferenced
                            let lhs_dereferenced_size_bytes = lhs.get_data_type().remove_outer_modifier().memory_size().size_bytes();
                            //convert this number to a string
                            let lhs_deref_size_str = NumberLiteral::try_new(&lhs_dereferenced_size_bytes.to_string()).unwrap().nasm_format();

                            asm_comment!(result, "lhs is a pointer. make rhs {} times bigger", lhs_deref_size_str);

                            assert!(promoted_type.memory_size().size_bytes() == 8);
                            asm_line!(result, "mov rcx, {}", lhs_deref_size_str);//get the size of value pointed to by rhs
                            asm_line!(result, "mul rcx");//multiply rhs by the size of value pointed to by lhs, so that +1 would skip along 1 value, not 1 byte
                            
                            //rhs now in AX
                        }

                        //pop lhs to secondary register, since rhs is already in acc
                        asm_line!(result, "{}", asm_boilerplate::pop_reg(&promoted_size, &LogicalRegister::SECONDARY));

                        asm_line!(result, "add {}, {}",
                            LogicalRegister::ACC.generate_reg_name(promoted_size),
                            LogicalRegister::SECONDARY.generate_reg_name(promoted_size)
                        );

                        //result is now in AX
                        
                    },
                    Punctuator::DASH => {
                        asm_comment!(result, "subtracting numbers");
                        asm_line!(result, "{}", put_lhs_ax_rhs_cx(&lhs, &rhs));

                        asm_line!(result, "sub {}, {}",
                        LogicalRegister::ACC.generate_reg_name(promoted_size),
                        LogicalRegister::SECONDARY.generate_reg_name(promoted_size)
                        );

                    }
                    Punctuator::ASTERISK => {
                        asm_comment!(result, "multiplying numbers");
                        asm_line!(result, "{}", put_lhs_ax_rhs_cx(&lhs, &rhs));

                        assert!(promoted_type.underlying_type().is_signed());//unsigned multiply??
                        assert!(promoted_type.underlying_type().is_integer());//floating point multiply??

                        asm_line!(result, "imul {}, {}",
                            LogicalRegister::ACC.generate_reg_name(promoted_size),
                            LogicalRegister::SECONDARY.generate_reg_name(promoted_size)
                        );

                    },
                    Punctuator::EQUALS => {//assign
                        asm_line!(result, "{}", generate_assembly_for_assignment(lhs, rhs, &promoted_type, promoted_size));
                    },
                    Punctuator::FORWARDSLASH => {
                        asm_comment!(result, "dividing numbers");
                        asm_line!(result, "{}", put_lhs_ax_rhs_cx(&lhs, &rhs));

                        match (promoted_type.memory_size().size_bytes(), promoted_type.underlying_type().is_signed()) {
                            (4,true) => {
                                asm_line!(result, "{}", asm_boilerplate::I32_DIVIDE_AX_BY_CX);
                            },
                            (8,true) => {
                                asm_line!(result, "{}", asm_boilerplate::I64_DIVIDE_AX_BY_CX);
                            }
                            _ => panic!("unsupported operands for divide")
                        }
                    },

                    Punctuator::PERCENT => {
                        asm_comment!(result, "calculating modulus");
                        asm_line!(result, "{}", put_lhs_ax_rhs_cx(&lhs, &rhs));

                        //modulus is calculated using a DIV
                        match (promoted_type.memory_size().size_bytes(), promoted_type.underlying_type().is_signed()) {
                            (4,true) => {
                                asm_line!(result, "{}", asm_boilerplate::I32_DIVIDE_AX_BY_CX);
                            },
                            (8,true) => {
                                asm_line!(result, "{}", asm_boilerplate::I64_DIVIDE_AX_BY_CX);
                            }
                            _ => panic!("unsupported operands")
                        }

                        //mod is returned in RDX
                        asm_line!(result, "{}", asm_boilerplate::mov_reg(&promoted_size, &LogicalRegister::ACC,  &PhysicalRegister::_DX));
                    }

                    comparison if comparison.as_comparator_instr().is_some() => { // >, <, ==, >=, <=
                        asm_comment!(result, "comparing numbers");
                        asm_line!(result, "{}", put_lhs_ax_rhs_cx(&lhs, &rhs));

                        let lhs_reg = LogicalRegister::ACC.generate_reg_name(promoted_size);
                        let rhs_reg = LogicalRegister::SECONDARY.generate_reg_name(promoted_size);

                        let result_size = MemoryLayout::from_bytes(1);
                        let result_reg = LogicalRegister::ACC;

                        asm_line!(result, "cmp {}, {}", lhs_reg, rhs_reg);//compare the two

                        asm_line!(result, "{} {}", comparison.as_comparator_instr().unwrap(), result_reg.generate_reg_name(&result_size));//create the correct set instruction

                    },

                    _ => panic!("operator to binary expression is invalid")
                }
            },
            Expression::FUNCCALL(call_data) => {
                asm_line!(result, "{}", call_data.generate_assembly());
            },
            Expression::STRINGLIT(_) => {
                asm_line!(result, "{}", self.put_lvalue_addr_in_acc());
            }
        };
        result
    }

    /**
     * put the address of myself in the accumulator
     */
    fn put_lvalue_addr_in_acc(&self) -> String {
        let mut result = String::new();

        match self {
            Expression::STACKVAR(decl) => {
                asm_comment!(result, "getting address of variable: {}", decl.decl.name);
                asm_line!(result, "lea rax, [rbp-{}]", decl.stack_offset.size_bytes());//calculate the address of the variable
            },
            Expression::PREFIXEXPR(Punctuator::ASTERISK, expr_box) => {
                //&*x == x
                asm_comment!(result, "getting address of a dereference");
                asm_line!(result, "{}", &expr_box.generate_assembly());
            },

            Expression::STRINGLIT(string_lit) => {
                asm_comment!(result, "getting address of string");
                asm_line!(result, "lea rax, [rel {}]", string_lit.get_label());
            }
            _ => panic!("tried to generate assembly to assign to a non-lvalue")
        };
        result
    }
}

/**
     * puts lhs in AX
     * puts rhs in CX
     * used in binary expressions, where you need both sides in registers
     * does NOT work for assignment expressions
     */
    fn put_lhs_ax_rhs_cx(lhs: &Expression, rhs: &Expression) -> String {
        let mut result = String::new();

        let promoted_type = DataType::calculate_promoted_type_arithmetic(&lhs.get_data_type(), &rhs.get_data_type());
        let promoted_size = &promoted_type.memory_size();

        //put lhs on stack
        asm_line!(result, "{}", lhs.generate_assembly());
        asm_line!(result, "{}", asm_boilerplate::cast_from_acc(&lhs.get_data_type(), &promoted_type));
        asm_line!(result, "{}", asm_boilerplate::push_reg(&lhs.get_data_type().memory_size(), &LogicalRegister::ACC));

        //put rhs in secondary
        asm_line!(result, "{}", rhs.generate_assembly());
        asm_line!(result, "{}", asm_boilerplate::cast_from_acc(&rhs.get_data_type(), &promoted_type));
        asm_line!(result, "{}", mov_reg(promoted_size, &LogicalRegister::SECONDARY, &LogicalRegister::ACC));//mov acc to secondary

        //pop lhs to ACC
        asm_line!(result, "{}", asm_boilerplate::pop_reg(&promoted_size, &LogicalRegister::ACC));

        result
    }

fn generate_assembly_for_assignment(lhs: &Expression, rhs: &Expression, promoted_type: &DataType, promoted_size: &MemoryLayout) -> String {
    let mut result = String::new();

    if lhs.get_data_type().is_array() && rhs.get_data_type().is_array() {
        //initialising an array? char[12] x = "hello world";//for example
        asm_line!(result, "{}", lhs.put_lvalue_addr_in_acc());//get dest address
        asm_line!(result, "{}", asm_boilerplate::push_reg(&PTR_SIZE, &LogicalRegister::ACC));//push to stack
        asm_line!(result, "{}", rhs.put_lvalue_addr_in_acc());//get src address
        asm_line!(result, "{}", asm_boilerplate::push_reg(&PTR_SIZE, &LogicalRegister::ACC));//push to stack

        asm_line!(result, "{}", asm_boilerplate::pop_reg(&PTR_SIZE, &PhysicalRegister::_SI));//pop source to RSI
        asm_line!(result, "{}", asm_boilerplate::pop_reg(&PTR_SIZE, &PhysicalRegister::_DI));//pop destination to RDI

        asm_line!(result, "mov rcx, {}", rhs.get_data_type().memory_size().size_bytes());//put number of bytes to copy in RCX

        asm_line!(result, "cld");//reset copy direction flag
        asm_line!(result, "rep movsb");//copy the data

        return result;//all done here
    }

    assert!(!lhs.get_data_type().is_array());
    assert!(lhs.get_data_type().memory_size().size_bits() <= 64);
    //maybe more special cases for pointer assignment etc

    //put address of lvalue on stack
    asm_line!(result, "{}", lhs.put_lvalue_addr_in_acc());
    asm_line!(result, "{}", asm_boilerplate::push_reg(&PTR_SIZE, &LogicalRegister::ACC));//push to stack
    
    //put the value to assign in acc
    asm_line!(result, "{}", rhs.generate_assembly());
    //cast to the same type as lhs
    asm_line!(result, "{}", asm_boilerplate::cast_from_acc(&rhs.get_data_type(), &promoted_type));

    asm_comment!(result, "assigning to a stack variable");

    //pop address to assign to
    asm_line!(result, "{}", asm_boilerplate::pop_reg(&PTR_SIZE, &LogicalRegister::SECONDARY));
    //save to memory
    asm_line!(result, "mov [{}], {}", LogicalRegister::SECONDARY.generate_reg_name(&PTR_SIZE), LogicalRegister::ACC.generate_reg_name(promoted_size));

    result
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
    let (left_part, right_part) = tokens_queue.split_to_slices(operator_idx.index, curr_queue_idx);

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