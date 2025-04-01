use unwrap_let::unwrap_let;

use crate::{asm_boilerplate::{self, mov_reg}, asm_gen_data::AsmData, asm_generation::{LogicalRegister, PhysicalRegister, RegisterName, PTR_SIZE}, ast_metadata::ASTMetadata, binary_expression::BinaryExpression, compilation_state::functions::FunctionList, data_type::recursive_data_type::RecursiveDataType, declaration::MinimalDataVariable, expression_visitors::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor, put_scalar_in_acc::ScalarInAccVisitor, reference_assembly_visitor::ReferenceVisitor}, function_call::FunctionCall, lexer::{precedence, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, number_literal::NumberLiteral, parse_data::ParseData, string_literal::StringLiteral, struct_definition::StructMemberAccess, unary_prefix_expr::UnaryPrefixExpression};
use std::fmt::Write;
use crate::asm_generation::{asm_line, asm_comment};

//none of these must reserve any stack space
#[derive(Clone)]
pub enum Expression {
    NUMBERLITERAL(NumberLiteral),
    VARIABLE(MinimalDataVariable),
    STRUCTMEMBERACCESS(StructMemberAccess),
    STRINGLITERAL(StringLiteral),
    FUNCCALL(FunctionCall),

    UNARYPREFIX(UnaryPrefixExpression),
    BINARYEXPRESSION(BinaryExpression),
}

impl Expression {

    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        match self {
            Expression::NUMBERLITERAL(x) => x.accept(visitor),
            Expression::VARIABLE(x) => x.accept(visitor),
            Expression::STRINGLITERAL(x) => x.accept(visitor),
            Expression::FUNCCALL(x) => x.accept(visitor),
            Expression::UNARYPREFIX(x) => x.accept(visitor),
            Expression::BINARYEXPRESSION(x) => x.accept(visitor),
            Expression::STRUCTMEMBERACCESS(x) => x.accept(visitor),
        }
    }
}

/**
 * tries to consume an expression, terminated by a semicolon, and returns None if this is not possible
 */
pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, scope_data: &ParseData) -> Option<ASTMetadata<Expression>> {
    let semicolon_idx = tokens_queue.find_closure_matches(&previous_queue_idx, false, |x| *x == Token::PUNCTUATOR(Punctuator::SEMICOLON), &TokenSearchType::skip_nothing())?;
    //define the slice that we are going to try and parse
    let attempt_slice = TokenQueueSlice {
        index: previous_queue_idx.index,
        max_index: semicolon_idx.index
    };

    match try_consume_whole_expr(tokens_queue, &attempt_slice, accessible_funcs, scope_data) {
        Some(expr) => {
            Some(ASTMetadata{resultant_tree: expr, remaining_slice: semicolon_idx.next_clone()})
        },
        None => None
    }
}
/**
 * tries to parse the tokens queue starting at previous_queue_idx, to find an expression
 * returns an expression(entirely consumed), else none
 */
pub fn try_consume_whole_expr(tokens_queue: &TokenQueue, previous_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, scope_data: &ParseData) -> Option<Expression> {
    let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

    println!("{:?}", tokens_queue.get_slice(previous_queue_idx));

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
            match tokens_queue.peek(& curr_queue_idx, &scope_data)? {
                Token::NUMBER(num) => {
                    tokens_queue.consume(&mut curr_queue_idx, &scope_data);
                    Some(Expression::NUMBERLITERAL(num))
                },
                Token::IDENTIFIER(var_name) => {
                    assert!(scope_data.variable_defined(&var_name));
                    Some(Expression::VARIABLE(MinimalDataVariable{name: var_name}))
                },
                Token::STRING(string_lit) => {
                    Some(Expression::STRINGLITERAL(string_lit))
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
                        if let Some(index_expr) = try_parse_array_index(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data) {
                            return Some(Expression::UNARYPREFIX(index_expr));
                        }

                        if let Some(func) = FunctionCall::try_consume_whole_expr(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data) {
                            return Some(Expression::FUNCCALL(func));
                        }

                        if let Some(access) = try_parse_member_access(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data) {
                            return Some(Expression::STRUCTMEMBERACCESS(access));
                        }
                    }

                } else {
                    //look for unary prefix as association is right to left
                    let first_token = tokens_queue.peek(&curr_queue_idx, &scope_data)?;

                    let starts_with_valid_prefix = first_token
                        .as_punctuator()
                        .and_then(|punc| punc.as_unary_prefix_precedence())
                        .is_some_and(|precedence| precedence == precedence_required);

                    if starts_with_valid_prefix {
                        match try_parse_unary_prefix(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data) {
                            Some(x) => {return Some(Expression::UNARYPREFIX(x));},
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
                    skip_in_squiggly_brackets: false
                };

                let operator_indexes = tokens_queue.split_by_closure_matches(&curr_queue_idx, associative_direction, operator_matching_closure, &exclusions);

                for operator_idx in operator_indexes {
                    //try to find an operator
                    //note that the operator_idx is a slice of just the operator

                    match try_parse_binary_expr(tokens_queue, &curr_queue_idx, &operator_idx, accessible_funcs, scope_data) {
                        Some(x) => {return Some(Expression::BINARYEXPRESSION(x));}
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
pub fn put_lhs_ax_rhs_cx(lhs: &Expression, rhs: &Expression, promoted_type: &RecursiveDataType, asm_data: &AsmData) -> String {
    let mut result = String::new();

    let promoted_size = promoted_type.memory_size(asm_data);

    //put lhs on stack
    let lhs_asm = lhs.accept(&mut ScalarInAccVisitor{asm_data});
    let lhs_type = lhs.accept(&mut GetDataTypeVisitor{asm_data});
    asm_line!(result, "{}", lhs_asm);
    asm_line!(result, "{}", asm_boilerplate::cast_from_acc(&lhs_type, &promoted_type, asm_data));
    asm_line!(result, "{}", asm_boilerplate::push_reg(&promoted_size, &LogicalRegister::ACC));

    //put rhs in secondary
    let rhs_asm = rhs.accept(&mut ScalarInAccVisitor{asm_data});
    let rhs_type = rhs.accept(&mut GetDataTypeVisitor{asm_data});
    asm_line!(result, "{}", rhs_asm);
    asm_line!(result, "{}", asm_boilerplate::cast_from_acc(&rhs_type, &promoted_type, asm_data));
    asm_line!(result, "{}", mov_reg(&promoted_size, &LogicalRegister::SECONDARY, &LogicalRegister::ACC));//mov acc to secondary

    //pop lhs to ACC
    asm_line!(result, "{}", asm_boilerplate::pop_reg(&promoted_size, &LogicalRegister::ACC));

    result
}

//TODO do I need promoted_type and promoted_size
pub fn generate_assembly_for_assignment(lhs: &Expression, rhs: &Expression, asm_data: &AsmData) -> String {
    let mut result = String::new();

    let promoted_type = lhs.accept(&mut GetDataTypeVisitor {asm_data});

    if let (RecursiveDataType::ARRAY {..}, RecursiveDataType::ARRAY {..}) = (&promoted_type, rhs.accept(&mut GetDataTypeVisitor {asm_data})){
        //promoted type and rhs are arrays
        //initialising an array? char[12] x = "hello world";//for example
        let lhs_asm = lhs.accept(&mut ReferenceVisitor {asm_data});
        let rhs_asm = rhs.accept(&mut ReferenceVisitor {asm_data});
        asm_line!(result, "{}", lhs_asm);//get dest address
        asm_line!(result, "{}", asm_boilerplate::push_reg(&PTR_SIZE, &LogicalRegister::ACC));//push to stack
        asm_line!(result, "{}", rhs_asm);//get src address
        asm_line!(result, "{}", asm_boilerplate::push_reg(&PTR_SIZE, &LogicalRegister::ACC));//push to stack

        asm_line!(result, "{}", asm_boilerplate::pop_reg(&PTR_SIZE, &PhysicalRegister::_SI));//pop source to RSI
        asm_line!(result, "{}", asm_boilerplate::pop_reg(&PTR_SIZE, &PhysicalRegister::_DI));//pop destination to RDI

        asm_line!(result, "mov rcx, {}", promoted_type.memory_size(asm_data).size_bytes());//put number of bytes to copy in RCX

        asm_line!(result, "cld");//reset copy direction flag
        asm_line!(result, "rep movsb");//copy the data

        return result;//all done here
    }

    match &promoted_type {
        RecursiveDataType::ARRAY {..} => {
            unwrap_let!(RecursiveDataType::ARRAY {..} = rhs.accept(&mut GetDataTypeVisitor{asm_data}));//rhs must be an array?

            //initialising an array? char[12] x = "hello world";//for example
            let lhs_asm = lhs.accept(&mut ReferenceVisitor {asm_data});
            let rhs_asm = rhs.accept(&mut ReferenceVisitor {asm_data});
            asm_line!(result, "{}", lhs_asm);//get dest address
            asm_line!(result, "{}", asm_boilerplate::push_reg(&PTR_SIZE, &LogicalRegister::ACC));//push to stack
            asm_line!(result, "{}", rhs_asm);//get src address
            asm_line!(result, "{}", asm_boilerplate::push_reg(&PTR_SIZE, &LogicalRegister::ACC));//push to stack

            asm_line!(result, "{}", asm_boilerplate::pop_reg(&PTR_SIZE, &PhysicalRegister::_SI));//pop source to RSI
            asm_line!(result, "{}", asm_boilerplate::pop_reg(&PTR_SIZE, &PhysicalRegister::_DI));//pop destination to RDI

            asm_line!(result, "mov rcx, {}", promoted_type.memory_size(asm_data).size_bytes());//put number of bytes to copy in RCX

            asm_line!(result, "cld");//reset copy direction flag
            asm_line!(result, "rep movsb");//copy the data
        },
        data_type => {
            assert!(data_type.memory_size(asm_data).size_bits() <= 64);
            //maybe more special cases for pointer assignment etc

            //put address of lvalue on stack
            let lhs_asm = lhs.accept(&mut ReferenceVisitor {asm_data});
            asm_line!(result, "{}", lhs_asm);
            asm_line!(result, "{}", asm_boilerplate::push_reg(&PTR_SIZE, &LogicalRegister::ACC));//push to stack
            
            //put the value to assign in acc
            let rhs_asm = rhs.accept(&mut ScalarInAccVisitor {asm_data});
            asm_line!(result, "{}", rhs_asm);
            //cast to the same type as lhs
            asm_line!(result, "{}", asm_boilerplate::cast_from_acc(&rhs.accept(&mut GetDataTypeVisitor{asm_data}), &promoted_type, asm_data));

            asm_comment!(result, "assigning to a stack variable");

            //pop address to assign to
            asm_line!(result, "{}", asm_boilerplate::pop_reg(&PTR_SIZE, &LogicalRegister::SECONDARY));
            //save to memory
            asm_line!(result, "mov [{}], {}", LogicalRegister::SECONDARY.generate_reg_name(&PTR_SIZE), LogicalRegister::ACC.generate_reg_name(&promoted_type.memory_size(asm_data)));
        },
    }

    result
}

/**
 * tries to parse the expression as a unary prefix and the operand, for example ++x or *(x->foo)
 * if the parse was successful, an expression is returned
 * else, you get None
 */
fn try_parse_unary_prefix(tokens_queue: &TokenQueue, previous_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, scope_data: &ParseData) -> Option<UnaryPrefixExpression> {
    let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);
    
    let punctuator = tokens_queue.consume(&mut curr_queue_idx, &scope_data)?.as_punctuator()?;//get punctuator

    punctuator.as_unary_prefix_precedence()?;//ensure the punctuator is a valid unary prefix

    let operand = try_consume_whole_expr(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data)?;

    Some(UnaryPrefixExpression::new(punctuator, operand))
}

/**
 * tries to parse the left and right hand side of operator_idx.index, as a binary expression e.g 1 + 2 split by "+"
 * if this parse was successful, an expression is returned
 * else, you get None
 */
fn try_parse_binary_expr(tokens_queue: &TokenQueue, curr_queue_idx: &TokenQueueSlice, operator_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, scope_data: &ParseData) -> Option<BinaryExpression> {
    //split to before and after the operator
    let (left_part, right_part) = tokens_queue.split_to_slices(operator_idx.index, curr_queue_idx);

    //try and parse the left and right hand sides, propogating errors
    let parsed_left = try_consume_whole_expr(tokens_queue, &left_part, accessible_funcs, scope_data)?;
    let parsed_right = try_consume_whole_expr(tokens_queue, &right_part, accessible_funcs, scope_data)?;

    let operator = tokens_queue.peek(&operator_idx, &scope_data).expect("couldn't peek")
        .as_punctuator().expect("couldn't cast to punctuator");

    Some(BinaryExpression::new(parsed_left, operator, parsed_right))
}

fn try_parse_array_index(tokens_queue: &TokenQueue, curr_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, scope_data: &ParseData) -> Option<UnaryPrefixExpression> {
    //look for unary postfixes as association is left to right
    let last_token = tokens_queue.peek_back(&curr_queue_idx, &scope_data)?;

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
            UnaryPrefixExpression::new(Punctuator::ASTERISK, //dereference
                Expression::BINARYEXPRESSION(BinaryExpression::new(array_expr, Punctuator::PLUS, index_expr))//pointer plus index
            )
        );
    }

    None
}

fn try_parse_member_access(tokens_queue: &TokenQueue, expr_slice: &TokenQueueSlice, accessible_funcs: &FunctionList, scope_data: &ParseData) -> Option<StructMemberAccess> {

    let mut curr_queue_idx = expr_slice.clone();

    assert!(tokens_queue.is_slice_inbounds(&curr_queue_idx));//ensure that the end of the slice is not infinite, so that I can decrement it to consume from the back
    
    let last_token = tokens_queue.peek_back(&curr_queue_idx, &scope_data)?;
    curr_queue_idx.max_index -= 1;//skip the member name at the back
    let penultimate_token = tokens_queue.peek_back(&curr_queue_idx, &scope_data)?;

    if penultimate_token != Token::PUNCTUATOR(Punctuator::FULLSTOP) {
        return None;//no fullstop to represent member access
    }

    curr_queue_idx.max_index -= 1;//skip the fullstop

    if let Token::IDENTIFIER(member_name) = last_token {//TODO what if a member name is the same as an enum variant
        //last token is a struct's member name
        //the first part must return a struct
        println!("struct tree: {:?}\nend of struct tree", tokens_queue.get_slice(&curr_queue_idx));
        let struct_tree = try_consume_whole_expr(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data)?;

        return Some(StructMemberAccess::new(struct_tree, member_name));
    }
    
    None//failed to find correct identifiers
}