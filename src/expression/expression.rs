use unwrap_let::unwrap_let;
use memory_size::MemorySize;
use crate::{ array_initialisation::ArrayInitialisation, asm_boilerplate::cast_from_acc, asm_gen_data::AsmData, assembly::{assembly::Assembly, operand::{immediate::MemorySizeExt, memory_operand::MemoryOperand, register::GPRegister, Operand, RegOrMem, PTR_SIZE}, operation::AsmOperation}, ast_metadata::ASTMetadata, binary_expression::BinaryExpression, cast_expr::CastExpression, compilation_state::label_generator::LabelGenerator, data_type::{base_type::BaseType, recursive_data_type::DataType}, debugging::ASTDisplay, declaration::MinimalDataVariable, expression::unary_prefix_expr::UnaryPrefixExpression, expression_visitors::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor, put_scalar_in_acc::ScalarInAccVisitor, reference_assembly_visitor::ReferenceVisitor}, function_call::FunctionCall, function_declaration::consume_fully_qualified_type, lexer::{keywords::Keyword, precedence, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, number_literal::typed_value::NumberLiteral, parse_data::ParseData, string_literal::StringLiteral, struct_member_access::StructMemberAccess};

use super::{binary_expression_operator::BinaryExpressionOperator, sizeof_expression::SizeofExpr, unary_postfix_expression::UnaryPostfixExpression, unary_postfix_operator::UnaryPostfixOperator, unary_prefix_operator::UnaryPrefixOperator};

#[derive(Clone, Debug)]
pub enum Expression {
    NUMBERLITERAL(NumberLiteral),
    VARIABLE(MinimalDataVariable),
    STRUCTMEMBERACCESS(StructMemberAccess),
    STRINGLITERAL(StringLiteral),//TODO merge with array initialisation
    ARRAYLITERAL(ArrayInitialisation),
    FUNCCALL(FunctionCall),

    UNARYPREFIX(UnaryPrefixExpression),
    UNARYSUFFIX(UnaryPostfixExpression),
    BINARYEXPRESSION(BinaryExpression),
    CAST(CastExpression),
    SIZEOF(SizeofExpr)
}

impl Expression {

    /**
     * tries to consume an expression, terminated by a semicolon, and returns None if this is not possible
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData, struct_label_gen: &mut LabelGenerator) -> Option<ASTMetadata<Expression>> {
        let semicolon_idx = tokens_queue.find_closure_matches(&previous_queue_idx, false, |x| *x == Token::PUNCTUATOR(Punctuator::SEMICOLON), &TokenSearchType::skip_all())?;
        //define the slice that we are going to try and parse
        let attempt_slice = TokenQueueSlice {
            index: previous_queue_idx.index,
            max_index: semicolon_idx.index
        };

        match try_consume_whole_expr(tokens_queue, &attempt_slice, scope_data, struct_label_gen) {
            Some(expr) => {
                Some(ASTMetadata{resultant_tree: expr, remaining_slice: semicolon_idx.next_clone()})
            },
            None => None
        }
    }

    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        match self {
            Expression::NUMBERLITERAL(x) => x.accept(visitor),
            Expression::VARIABLE(x) => x.accept(visitor),
            Expression::STRINGLITERAL(x) => x.accept(visitor),
            Expression::FUNCCALL(x) => x.accept(visitor),
            Expression::UNARYPREFIX(x) => x.accept(visitor),
            Expression::UNARYSUFFIX(x) => x.accept(visitor),
            Expression::BINARYEXPRESSION(x) => x.accept(visitor),
            Expression::STRUCTMEMBERACCESS(x) => x.accept(visitor),
            Expression::CAST(cast_expression) => cast_expression.accept(visitor),
            Expression::ARRAYLITERAL(_) => panic!("cannot determine data type/assemebly for array literal, try looking for casts or array initialisation instead"),
            Expression::SIZEOF(sizeof_expr) => sizeof_expr.accept(visitor),
        }
    }
}

/**
 * tries to parse the tokens queue starting at previous_queue_idx, to find an expression
 * returns an expression(entirely consumed), else none
 */
pub fn try_consume_whole_expr(tokens_queue: &TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData, struct_label_gen: &mut LabelGenerator) -> Option<Expression> {
    let mut curr_queue_idx = TokenQueueSlice {
        index: previous_queue_idx.index,
        max_index: previous_queue_idx.max_index.min(tokens_queue.tokens.len())//prevent very wide slices from overflowing the array
    };

    if tokens_queue.slice_is_brackets(&curr_queue_idx, Punctuator::OPENCURLY) {
        //we are an expression surrounded by brackets
        //remove the outer brackets and continue
        curr_queue_idx = TokenQueueSlice {
            index: curr_queue_idx.index+1,
            max_index: curr_queue_idx.max_index-1
        };
    }

    //look for array initialisation
    if let Some(x) = ArrayInitialisation::try_consume_whole_expr(tokens_queue, previous_queue_idx, scope_data, struct_label_gen) {
        return Some(Expression::ARRAYLITERAL(x));
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
                        if let Some(index_expr) = try_parse_array_index(tokens_queue, &curr_queue_idx, scope_data, struct_label_gen) {
                            return Some(Expression::UNARYPREFIX(index_expr));//since a[b] = *(a+b), indexing returns a unary prefix
                        }

                        if let Some(func) = FunctionCall::try_consume_whole_expr(tokens_queue, &curr_queue_idx, scope_data, struct_label_gen) {
                            return Some(Expression::FUNCCALL(func));
                        }

                        if let Some(access) = try_parse_member_access(tokens_queue, &curr_queue_idx, scope_data, struct_label_gen) {
                            return Some(Expression::STRUCTMEMBERACCESS(access));
                        }
                    }

                    let last_token = tokens_queue.peek_back(&curr_queue_idx, &scope_data).unwrap();
                    let ends_with_valid_suffix = last_token
                        .as_punctuator()
                        .and_then(|punc| punc.as_unary_suffix_precendece())
                        .is_some_and(|precedence| precedence == precedence_required);

                    if ends_with_valid_suffix {
                        if let Some(x) = try_parse_unary_suffix(tokens_queue, &curr_queue_idx, scope_data, struct_label_gen) {
                            return Some(Expression::UNARYSUFFIX(x));
                        }
                    }

                } else {
                    //look for unary prefix as association is right to left
                    let first_token = tokens_queue.peek(&curr_queue_idx, &scope_data)?;

                    let starts_with_valid_prefix = first_token
                        .as_punctuator()
                        .and_then(|punc| punc.as_unary_prefix_precedence())
                        .is_some_and(|precedence| precedence == precedence_required);

                    if starts_with_valid_prefix {//TODO do I need this if statement
                        if let Some(x) = try_parse_unary_prefix(tokens_queue, &curr_queue_idx, scope_data, struct_label_gen) {
                            return Some(Expression::UNARYPREFIX(x));
                        }
                    }

                    if precedence_required == 2 {
                        //parse cast expression
                        if let Some(cast) = try_parse_cast(tokens_queue, &curr_queue_idx, scope_data, struct_label_gen) {
                            return Some(Expression::CAST(cast));
                        }
                        if let Some(sizeof_expr) = try_parse_sizeof(tokens_queue, &curr_queue_idx, scope_data, struct_label_gen) {
                            return Some(Expression::SIZEOF(sizeof_expr));
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

                    match try_parse_binary_expr(tokens_queue, &curr_queue_idx, operator_idx, scope_data, struct_label_gen) {
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
 * puts lhs in AX, casted to lhs_new_type
 * puts rhs in CX, casted to rhs_new_type
 * used in binary expressions, where you need both sides in registers
 * does NOT work for assignment expressions
 */
pub fn put_lhs_ax_rhs_cx(lhs: &Expression, lhs_new_type: &DataType, rhs: &Expression, rhs_new_type: &DataType, asm_data: &AsmData, stack_data: &mut MemorySize) -> Assembly {
    let mut result = Assembly::make_empty();

    //put rhs in on the stack
    let rhs_new_size = rhs_new_type.memory_size(asm_data);
    let rhs_asm = rhs.accept(&mut ScalarInAccVisitor{asm_data, stack_data});
    let rhs_type = rhs.accept(&mut GetDataTypeVisitor{asm_data});
    let rhs_cast_asm = cast_from_acc(&rhs_type, &rhs_new_type, asm_data);
    result.merge(&rhs_asm);
    result.merge(&rhs_cast_asm);
    
    *stack_data += rhs_new_size;//allocate temporary storage
    let rhs_temporary_address = stack_data.clone();
    result.add_instruction(AsmOperation::MOV {
        to: RegOrMem::Mem(MemoryOperand::SubFromBP(rhs_temporary_address)),
        from: Operand::Reg(GPRegister::acc()),
        size: rhs_new_size,
    });

    //put lhs in ACC
    let lhs_asm = lhs.accept(&mut ScalarInAccVisitor{asm_data, stack_data});
    let lhs_type = lhs.accept(&mut GetDataTypeVisitor{asm_data});
    let lhs_cast_asm = cast_from_acc(&lhs_type, lhs_new_type, asm_data);
    result.merge(&lhs_asm);
    result.merge(&lhs_cast_asm);

    //read rhs to secondary
    result.add_instruction(AsmOperation::MOV {
        to: RegOrMem::Reg(GPRegister::secondary()),
        from: Operand::Mem(MemoryOperand::SubFromBP(rhs_temporary_address)),
        size: rhs_new_size,
    });

    result
}

pub fn generate_assembly_for_assignment(lhs: &Expression, rhs: &Expression, asm_data: &AsmData, stack_data: &mut MemorySize) -> Assembly {
    let mut result = Assembly::make_empty();

    let promoted_type = lhs.accept(&mut GetDataTypeVisitor {asm_data});

    match (&promoted_type, rhs) {
        //initialising array to string literal
        (DataType::ARRAY {..}, Expression::STRINGLITERAL(_)) => {
            let lhs_addr_asm = lhs.accept(&mut ReferenceVisitor {asm_data, stack_data});
            let rhs_addr_asm = rhs.accept(&mut ReferenceVisitor {asm_data, stack_data});
            result.merge(&lhs_addr_asm);//get dest address

            *stack_data += PTR_SIZE;//allocate temporary storage for destination address
            let destination_temporary_storage = stack_data.clone();
            //store the destination address in a temporary stack variable
            result.add_instruction(AsmOperation::MOV {
                to: RegOrMem::Mem(MemoryOperand::SubFromBP(destination_temporary_storage)),
                from: Operand::Reg(GPRegister::acc()),
                size: PTR_SIZE,
            });

            result.merge(&rhs_addr_asm);//get src address

            result.add_instruction(AsmOperation::MOV {
                to: RegOrMem::Reg(GPRegister::_SI),
                from: Operand::Reg(GPRegister::acc()),
                size: PTR_SIZE,
            });
            result.add_instruction(AsmOperation::MOV {
                to: RegOrMem::Reg(GPRegister::_DI),
                from: Operand::Mem(MemoryOperand::SubFromBP(destination_temporary_storage)),
                size: PTR_SIZE,
            });

            result.add_instruction(AsmOperation::MEMCPY { size: promoted_type.memory_size(asm_data) });
        },

        //initialising array to array literal
        (DataType::ARRAY { .. }, Expression::ARRAYLITERAL(array_init)) => {

            //convert int x[2][2] to int x[4] for easy assigning of values
            unwrap_let!(DataType::ARRAY { element: array_element_type, .. } = promoted_type.flatten_nested_array());
            let array_element_size = array_element_type.memory_size(asm_data);

            //get address of destination array
            let lhs_addr_asm = lhs.accept(&mut ReferenceVisitor {asm_data, stack_data});
            result.merge(&lhs_addr_asm);

            //store lhs address on stack
            *stack_data += PTR_SIZE;
            let lhs_addr_storage = stack_data.clone();
            result.add_instruction(AsmOperation::MOV {
                to: RegOrMem::Mem(MemoryOperand::SubFromBP(lhs_addr_storage)),
                from: Operand::Reg(GPRegister::acc()),
                size: PTR_SIZE,
            });

            //this generates the following c-style code to assign the array literal to the destination array
            //for 2d arrays, this code reinteprets it as a 1d array, using zero_fill_and_flatten_to_iter which flattens to 1d array
            //for(int i=0;i<array_size;i++){
            //  array[i] = array_literal[i];
            //}
            for (i, item) in array_init.zero_fill_and_flatten_to_iter(&promoted_type).iter().enumerate() {

                //generate the item
                result.merge(&item.accept(&mut ScalarInAccVisitor{asm_data, stack_data}));

                //get address of the start of the array
                result.add_instruction(AsmOperation::MOV {
                    to: RegOrMem::Reg(GPRegister::secondary()),
                    from: Operand::Mem(MemoryOperand::SubFromBP(lhs_addr_storage)),
                    size: PTR_SIZE,
                });

                let array_start_offset = MemorySize::from_bytes(i as u64 * array_element_size.size_bytes());//how many bytes from the start of the array is the item
                //add the index to it: (void*)ndarray + i
                result.add_instruction(AsmOperation::ADD {
                    destination: RegOrMem::Reg(GPRegister::secondary()),
                    increment: Operand::Imm(array_start_offset.as_imm()),
                    data_type: DataType::RAW(BaseType::U64),
                });

                result.add_commented_instruction(AsmOperation::MOV {
                    to: RegOrMem::Mem(MemoryOperand::MemoryAddress { pointer_reg: GPRegister::secondary() }),
                    from: Operand::Reg(GPRegister::acc()),
                    size: array_element_size,
                }, format!("initialising element {} of array", i));
            }
        },

        (DataType::ARRAY { .. }, x) => panic!("tried to set {:?} to {:?}", lhs, x),

        (data_type, _) => {
            assert!(data_type.memory_size(asm_data).size_bytes() <= 8);
            //maybe more special cases for struct assignment etc

            //put address of lvalue on stack
            let lhs_asm = lhs.accept(&mut ReferenceVisitor {asm_data, stack_data});
            result.merge(&lhs_asm);

            *stack_data += PTR_SIZE;//allocate temporary lhs storage
            let lhs_temporary_address = stack_data.clone();
            result.add_instruction(AsmOperation::MOV {
                to: RegOrMem::Mem(MemoryOperand::SubFromBP(lhs_temporary_address)),
                from: Operand::Reg(GPRegister::acc()),
                size: PTR_SIZE,
            });
            
            //put the value to assign in acc, and cast to correct type
            let rhs_asm = rhs.accept(&mut ScalarInAccVisitor {asm_data, stack_data});
            let rhs_cast_asm = cast_from_acc(&rhs.accept(&mut GetDataTypeVisitor{asm_data}), &promoted_type, asm_data);
            result.merge(&rhs_asm);
            result.merge(&rhs_cast_asm);

            result.add_comment("assigning to a stack variable");

            //read lhs as address to assign to
            result.add_instruction(AsmOperation::MOV {
                to: RegOrMem::Reg(GPRegister::secondary()),
                from: Operand::Mem(MemoryOperand::SubFromBP(lhs_temporary_address)),
                size: PTR_SIZE,
            });

            //save to memory
            result.add_instruction(AsmOperation::MOV {
                to: RegOrMem::Mem(MemoryOperand::MemoryAddress {pointer_reg: GPRegister::secondary()} ),
                from: Operand::Reg(GPRegister::acc()), 
                size: promoted_type.memory_size(asm_data)
            });
        },
    }

    result
}

/**
 * tries to parse the expression as a unary prefix and the operand, for example ++x or *(x->foo)
 * if the parse was successful, an expression is returned
 * else, you get None
 */
fn try_parse_unary_prefix(tokens_queue: &TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData, struct_label_gen: &mut LabelGenerator) -> Option<UnaryPrefixExpression> {
    let mut curr_queue_idx = previous_queue_idx.clone();
    
    let unary_op: UnaryPrefixOperator = tokens_queue.consume(&mut curr_queue_idx, &scope_data)
    .and_then(|tok| tok.as_punctuator())
    .and_then(|punc| punc.try_into().ok())?;//get unary operator, or return if it isn't one

    let operand = try_consume_whole_expr(tokens_queue, &curr_queue_idx, scope_data, struct_label_gen)?;

    Some(UnaryPrefixExpression::new(unary_op, operand))
}

fn try_parse_unary_suffix(tokens_queue: &TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData, struct_label_gen: &mut LabelGenerator) -> Option<UnaryPostfixExpression> {
    let mut curr_queue_idx = previous_queue_idx.clone();
    
    let unary_op: UnaryPostfixOperator = tokens_queue.peek_back(&curr_queue_idx, &scope_data)
    .and_then(|tok| tok.as_punctuator())
    .and_then(|punc| punc.try_into().ok())?;//get unary operator, or return if it isn't one

    curr_queue_idx.max_index -= 1;//consume the last token

    let operand = try_consume_whole_expr(tokens_queue, &curr_queue_idx, scope_data, struct_label_gen)?;

    Some(UnaryPostfixExpression::new(unary_op, operand))
}

/**
 * tries to parse the left and right hand side of operator_idx, as a binary expression e.g 1 + 2 split by "+"
 * if this parse was successful, an expression is returned
 * else, you get None
 */
fn try_parse_binary_expr(tokens_queue: &TokenQueue, curr_queue_idx: &TokenQueueSlice, operator_idx: usize, scope_data: &mut ParseData, struct_label_gen: &mut LabelGenerator) -> Option<BinaryExpression> {
    //split to before and after the operator
    let (left_part, right_part) = tokens_queue.split_to_slices(operator_idx, curr_queue_idx);

    //try and parse the left and right hand sides, propogating errors
    let parsed_left = try_consume_whole_expr(tokens_queue, &left_part, scope_data, struct_label_gen)?;
    let parsed_right = try_consume_whole_expr(tokens_queue, &right_part, scope_data, struct_label_gen)?;

    let operator = tokens_queue.peek(&TokenQueueSlice { index: operator_idx, max_index: operator_idx+1 }, &scope_data)//get token in the middle
    .and_then(|x| x.as_punctuator())//try to convert to punctuator
    .and_then(|x| x.try_into().ok())?;//try to convert to binary expression operator

    Some(BinaryExpression::new(parsed_left, operator, parsed_right))
}

fn try_parse_array_index(tokens_queue: &TokenQueue, curr_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData, struct_label_gen: &mut LabelGenerator) -> Option<UnaryPrefixExpression> {
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

        let index_expr = try_consume_whole_expr(tokens_queue, &index_slice, scope_data, struct_label_gen)?;
        let array_expr = try_consume_whole_expr(tokens_queue, &array_slice, scope_data, struct_label_gen)?;

        //a[b] == *(a+b) in C
        return Some(
            UnaryPrefixExpression::new(UnaryPrefixOperator::Dereference, //dereference
                Expression::BINARYEXPRESSION(BinaryExpression::new(array_expr, BinaryExpressionOperator::Add, index_expr))//pointer plus index
            )
        );
    }

    None
}

fn try_parse_member_access(tokens_queue: &TokenQueue, expr_slice: &TokenQueueSlice, scope_data: &mut ParseData, struct_label_gen: &mut LabelGenerator) -> Option<StructMemberAccess> {

    let mut curr_queue_idx = expr_slice.clone();

    assert!(tokens_queue.is_slice_inbounds(&curr_queue_idx));//ensure that the end of the slice is not infinite, so that I can decrement it to consume from the back
    
    let last_token = tokens_queue.peek_back(&curr_queue_idx, &scope_data)?;
    curr_queue_idx.max_index -= 1;//skip the member name at the back
    let penultimate_token = tokens_queue.peek_back(&curr_queue_idx, &scope_data)?;

    if penultimate_token != Token::PUNCTUATOR(Punctuator::FULLSTOP) {
        return None;//no fullstop to represent member access
    }

    curr_queue_idx.max_index -= 1;//skip the fullstop

    if let Token::IDENTIFIER(member_name) = last_token {
        //last token is a struct's member name
        //the first part must return a struct
        let struct_tree = try_consume_whole_expr(tokens_queue, &curr_queue_idx, scope_data, struct_label_gen)?;

        return Some(StructMemberAccess::new(struct_tree, member_name));
    }
    
    None//failed to find correct identifiers
}

fn try_parse_sizeof(tokens_queue: &TokenQueue, expr_slice: &TokenQueueSlice, scope_data: &mut ParseData, struct_label_gen: &mut LabelGenerator) -> Option<SizeofExpr> {
    let mut curr_queue_idx = expr_slice.clone();

    if tokens_queue.consume(&mut curr_queue_idx, scope_data)? != Token::KEYWORD(Keyword::SIZEOF) {
        return None;//must start with sizeof
    }

    if tokens_queue.slice_is_brackets(&curr_queue_idx, Punctuator::OPENCURLY) {
        //go inside brackets if they are present
        curr_queue_idx.index += 1;
        curr_queue_idx.max_index -= 1;
    }

    //try and look for an expression
    let base_expr = try_consume_whole_expr(tokens_queue, &curr_queue_idx, scope_data, struct_label_gen);
    //try and look for a data type
    let data_type = consume_fully_qualified_type(tokens_queue, &curr_queue_idx, scope_data, struct_label_gen)
        .map(|x| {
            assert!(x.remaining_slice.get_slice_size() == 0);
            x.resultant_tree.0
        });

    match (base_expr, data_type) {
        (None, None) => None,
        (_, Some(x)) => Some(SizeofExpr::SizeofType(x)),//in one test case, int8_t was being considered a variable and a type, so just consider it as a type here
        (Some(x), None) => Some(SizeofExpr::SizeofExpression(Box::new(x))),
    }
}

fn try_parse_cast(tokens_queue: &TokenQueue, expr_slice: &TokenQueueSlice, scope_data: &mut ParseData, struct_label_gen: &mut LabelGenerator) -> Option<CastExpression> {
    let mut curr_queue_idx = expr_slice.clone();

    if tokens_queue.consume(&mut curr_queue_idx, scope_data)? != Token::PUNCTUATOR(Punctuator::OPENCURLY) {
        return None;//cast must start with "("
    }

    //match closure, as no nested brackets allowed in cast type
    let close_curly_idx = tokens_queue.find_closure_matches(&curr_queue_idx, false, |x| *x == Token::PUNCTUATOR(Punctuator::CLOSECURLY), &TokenSearchType::skip_nothing())?.index;

    //split the token slice like: ( dtype ) expr  ->  dtype, expr
    let new_type_slice = TokenQueueSlice {
        index: curr_queue_idx.index,
        max_index: close_curly_idx,
    };
    let remaining_expr_slice = TokenQueueSlice {
        index: close_curly_idx+1,
        max_index: curr_queue_idx.max_index,
    };

    //discard storage duration for cast
    let ASTMetadata { remaining_slice, resultant_tree: (new_type, _) } = consume_fully_qualified_type(tokens_queue, &new_type_slice, scope_data, struct_label_gen)?;
    assert!(remaining_slice.get_slice_size() == 0);//cannot be any remaining tokens in the cast type

    let base_expr = try_consume_whole_expr(tokens_queue, &remaining_expr_slice, scope_data, struct_label_gen)?;

    Some(CastExpression::new(new_type, base_expr))
}

impl ASTDisplay for Expression {
    fn display_ast(&self, f: &mut crate::debugging::TreeDisplayInfo) {
        match self {
            Expression::NUMBERLITERAL(number_literal) => f.write(&format!("{}", number_literal)),
            Expression::VARIABLE(minimal_data_variable) => f.write(&format!("{}", minimal_data_variable)),
            Expression::STRUCTMEMBERACCESS(struct_member_access) => struct_member_access.display_ast(f),
            Expression::STRINGLITERAL(string_literal) => f.write(&format!("{}", string_literal)),
            Expression::ARRAYLITERAL(array_initialisation) => array_initialisation.display_ast(f),
            Expression::FUNCCALL(function_call) => function_call.display_ast(f),
            Expression::UNARYPREFIX(unary_prefix_expression) => unary_prefix_expression.display_ast(f),
            Expression::UNARYSUFFIX(unary_suffix) => unary_suffix.display_ast(f),
            Expression::BINARYEXPRESSION(binary_expression) => binary_expression.display_ast(f),
            Expression::CAST(cast_expression) => cast_expression.display_ast(f),
            Expression::SIZEOF(sizeof_expr) => sizeof_expr.display_ast(f),
        }
    }
}