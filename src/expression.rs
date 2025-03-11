use crate::{asm_boilerplate::{self, mov_reg}, asm_generation::{LogicalRegister, PhysicalRegister, RegisterName, PTR_SIZE}, ast_metadata::ASTMetadata, binary_expression::BinaryExpression, compilation_state::functions::FunctionList, data_type::data_type::DataType, function_call::FunctionCall, lexer::{precedence, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, memory_size::MemoryLayout, scope_data::ScopeData, unary_prefix_expr::UnaryPrefixExpression};
use std::fmt::Write;
use crate::asm_generation::{asm_line, asm_comment};

pub trait ExprNode {
    fn generate_assembly(&self) -> String;
    fn get_data_type(&self) -> DataType;
    fn put_lvalue_addr_in_acc(&self) -> String;
}

/**
 * tries to consume an expression, terminated by a semicolon, and returns None if this is not possible
 */
pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, scope_data: &mut ScopeData) -> Option<ASTMetadata<Box<dyn ExprNode>>> {
    let semicolon_idx = tokens_queue.find_closure_in_slice(&previous_queue_idx, false, |x| *x == Token::PUNCTUATOR(Punctuator::SEMICOLON))?;
    //define the slice that we are going to try and parse
    let attempt_slice = TokenQueueSlice {
        index: previous_queue_idx.index,
        max_index: semicolon_idx.index
    };

    match try_consume_whole_expr(tokens_queue, &attempt_slice, accessible_funcs, scope_data) {
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
pub fn try_consume_whole_expr(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, scope_data: &mut ScopeData) -> Option<Box<dyn ExprNode>> {
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
                    Some(Box::new(num))
                },
                Token::IDENTIFIER(var_name) => {
                    Some(Box::new(scope_data.stack_vars.get_variable(&var_name).unwrap()))
                },
                Token::STRING(string_lit) => {
                    Some(Box::new(string_lit))
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
                        match try_parse_array_index(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data) {
                            Some(x) => {return Some(x)},
                            None => {}
                        }

                        if let Some(func) = FunctionCall::try_consume_whole_expr(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data) {
                            return Some(Box::new(func));
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
                        match try_parse_unary_prefix(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data) {
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

                    match try_parse_binary_expr(tokens_queue, &curr_queue_idx, &operator_idx, accessible_funcs, scope_data) {
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

/**
 * puts lhs in AX
 * puts rhs in CX
 * used in binary expressions, where you need both sides in registers
 * does NOT work for assignment expressions
 */
pub fn put_lhs_ax_rhs_cx(lhs: &dyn ExprNode, rhs: &dyn ExprNode) -> String {
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

//TODO do I need promoted_type and promoted_size
pub fn generate_assembly_for_assignment(lhs: &dyn ExprNode, rhs: &dyn ExprNode, promoted_type: &DataType, promoted_size: &MemoryLayout) -> String {
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
fn try_parse_unary_prefix(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, scope_data: &mut ScopeData) -> Option<Box<dyn ExprNode>> {
    let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);
    
    let punctuator = tokens_queue.consume(&mut curr_queue_idx)?.as_punctuator()?;//get punctuator

    punctuator.as_unary_prefix_precedence()?;//ensure the punctuator is a valid unary prefix

    let operand = try_consume_whole_expr(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data)?;

    Some(Box::new(UnaryPrefixExpression::new(punctuator, operand)))
}

/**
 * tries to parse the left and right hand side of operator_idx.index, as a binary expression e.g 1 + 2 split by "+"
 * if this parse was successful, an expression is returned
 * else, you get None
 */
fn try_parse_binary_expr(tokens_queue: &mut TokenQueue, curr_queue_idx: &TokenQueueSlice, operator_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, scope_data: &mut ScopeData) -> Option<Box<dyn ExprNode>> {
    //split to before and after the operator
    let (left_part, right_part) = tokens_queue.split_to_slices(operator_idx.index, curr_queue_idx);

    //try and parse the left and right hand sides, propogating errors
    let parsed_left = try_consume_whole_expr(tokens_queue, &left_part, accessible_funcs, scope_data)?;
    let parsed_right = try_consume_whole_expr(tokens_queue, &right_part, accessible_funcs, scope_data)?;

    let operator = tokens_queue.peek(&operator_idx).expect("couldn't peek")
        .as_punctuator().expect("couldn't cast to punctuator");

    Some(Box::new(BinaryExpression::new(parsed_left, operator, parsed_right)))
}

fn try_parse_array_index(tokens_queue: &mut TokenQueue, curr_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, scope_data: &mut ScopeData) -> Option<Box<dyn ExprNode>> {
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

        let index_expr = try_consume_whole_expr(tokens_queue, &index_slice, accessible_funcs, scope_data)?;
        let array_expr = try_consume_whole_expr(tokens_queue, &array_slice, accessible_funcs, scope_data)?;

        //a[b] == *(a+b) in C
        return Some(
            Box::new(UnaryPrefixExpression::new(Punctuator::ASTERISK, Box::new(//dereference
                BinaryExpression::new(array_expr, Punctuator::PLUS, index_expr)//pointer plus index
            )))
        );
    }

    None
}