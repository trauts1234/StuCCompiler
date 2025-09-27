use stack_management::{simple_stack_frame::SimpleStackFrame, stack_item::StackItemKey};
use unwrap_let::unwrap_let;
use crate::{ array_initialisation::ArrayInitialisation, asm_gen_data::{AsmData, GetStructUnion, GlobalAsmData}, assembly::{assembly::IRCode, operand::{immediate::ToImmediate, Storage, PTR_SIZE}, operation::IROperation}, ast_metadata::ASTMetadata, binary_expression::BinaryExpression, cast_expr::CastExpression, data_type::{base_type::{BaseType, IntegerType, ScalarType}, recursive_data_type::DataType}, debugging::ASTDisplay, declaration::MinimalDataVariable, expression::{ternary::TernaryExpr, unary_prefix_expr::UnaryPrefixExpression}, expression_visitors::expr_visitor::ExprVisitor, function_call::FunctionCall, function_declaration::consume_fully_qualified_type, generate_ir_traits::{GenerateIR, GetAddress, GetType}, lexer::{keywords::Keyword, precedence, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, member_access::MemberAccess, number_literal::typed_value::NumberLiteral, parse_data::ParseData, string_literal::StringLiteral};

use super::{binary_expression_operator::BinaryExpressionOperator, sizeof_expression::SizeofExpr, unary_postfix_expression::UnaryPostfixExpression, unary_postfix_operator::UnaryPostfixOperator, unary_prefix_operator::UnaryPrefixOperator};

#[derive(Clone, Debug)]
pub enum Expression {
    NUMBERLITERAL(NumberLiteral),
    VARIABLE(MinimalDataVariable),
    STRUCTMEMBERACCESS(MemberAccess),
    STRINGLITERAL(StringLiteral),//TODO merge with array initialisation
    ARRAYLITERAL(ArrayInitialisation),
    FUNCCALL(FunctionCall),

    UNARYPREFIX(UnaryPrefixExpression),
    UNARYSUFFIX(UnaryPostfixExpression),
    BINARYEXPRESSION(BinaryExpression),
    TERNARYEXPRESSION(TernaryExpr),
    CAST(CastExpression),
    SIZEOF(SizeofExpr)
}

impl Expression {

    /**
     * tries to consume an expression, terminated by a semicolon, and returns None if this is not possible
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<ASTMetadata<Expression>> {
        let semicolon_idx = tokens_queue.find_closure_matches(&previous_queue_idx, false, |x| *x == Token::PUNCTUATOR(Punctuator::SEMICOLON), &TokenSearchType::skip_all_brackets())?;
        //define the slice that we are going to try and parse
        let attempt_slice = TokenQueueSlice {
            index: previous_queue_idx.index,
            max_index: semicolon_idx
        };

        match try_consume_whole_expr(tokens_queue, &attempt_slice, scope_data) {
            Some(expr) => {
                Some(ASTMetadata{resultant_tree: expr, remaining_slice: TokenQueueSlice { index: semicolon_idx+1, max_index: previous_queue_idx.max_index }})
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
            Expression::ARRAYLITERAL(x) => panic!("cannot determine data type/assemebly for array literal, try looking for casts or array initialisation instead\nfor array {:?}", x),
            Expression::SIZEOF(sizeof_expr) => sizeof_expr.accept(visitor),
            Expression::TERNARYEXPRESSION(x) => x.accept(visitor),
        }
    }
}

impl GenerateIR for Expression {
    fn generate_ir(&self, asm_data: &AsmData, stack_data: &mut SimpleStackFrame, global_asm_data: &GlobalAsmData) -> (IRCode, Option<StackItemKey>) {
        let mut result = IRCode::make_empty();

        match self {
            Expression::NUMBERLITERAL(number_literal) => number_literal.generate_ir(asm_data, stack_data, global_asm_data),
            Expression::VARIABLE(minimal_data_variable) => minimal_data_variable.generate_ir(asm_data, stack_data, global_asm_data),
            Expression::STRUCTMEMBERACCESS(member_access) => todo!(),
            Expression::STRINGLITERAL(string_literal) => todo!(),
            Expression::ARRAYLITERAL(array_initialisation) => todo!(),
            Expression::FUNCCALL(function_call) => function_call.generate_ir(asm_data, stack_data, global_asm_data),
            Expression::UNARYPREFIX(unary_prefix_expression) => unary_prefix_expression.generate_ir(asm_data, stack_data, global_asm_data),
            Expression::UNARYSUFFIX(unary_postfix_expression) => unary_postfix_expression.generate_ir(asm_data, stack_data, global_asm_data),
            Expression::BINARYEXPRESSION(binary_expression) => binary_expression.generate_ir(asm_data, stack_data, global_asm_data),
            Expression::TERNARYEXPRESSION(ternary_expr) => todo!(),
            Expression::CAST(cast_expression) => todo!(),
            Expression::SIZEOF(sizeof_expr) => todo!(),
        }
    }
}
impl GetType for Expression {
    fn get_type(&self, asm_data: &AsmData) -> DataType {
        match self {
            Expression::NUMBERLITERAL(number_literal) => DataType::RAW(BaseType::Scalar(number_literal.get_data_type())),
            Expression::VARIABLE(minimal_data_variable) => minimal_data_variable.get_type(asm_data),
            Expression::STRUCTMEMBERACCESS(member_access) => member_access.get_type(asm_data),
            Expression::STRINGLITERAL(string_literal) => string_literal.get_type(asm_data),
            Expression::ARRAYLITERAL(array_initialisation) => array_initialisation.get_type(asm_data),
            Expression::FUNCCALL(function_call) => function_call.get_type(asm_data),
            Expression::UNARYPREFIX(unary_prefix_expression) => unary_prefix_expression.get_type(asm_data),
            Expression::UNARYSUFFIX(unary_postfix_expression) => unary_postfix_expression.get_type(asm_data),
            Expression::BINARYEXPRESSION(binary_expression) => binary_expression.get_type(asm_data),
            Expression::TERNARYEXPRESSION(ternary_expr) => ternary_expr.get_type(asm_data),
            Expression::CAST(cast_expression) => cast_expression.get_type(asm_data),
            Expression::SIZEOF(sizeof_expr) => sizeof_expr.get_type(asm_data),
        }
    }
}
impl GetAddress for Expression {
    fn get_address(&self, asm_data: &AsmData, stack_data: &mut SimpleStackFrame, global_asm_data: &GlobalAsmData) -> (IRCode, StackItemKey) {
        match self {
            Expression::NUMBERLITERAL(number_literal) => panic!("can't get address of number"),
            Expression::VARIABLE(minimal_data_variable) => minimal_data_variable.get_address(asm_data, stack_data, global_asm_data),
            Expression::STRUCTMEMBERACCESS(member_access) => todo!(),
            Expression::STRINGLITERAL(string_literal) => todo!(),
            Expression::ARRAYLITERAL(array_initialisation) => panic!("can't get address of array literal"),
            Expression::FUNCCALL(function_call) => todo!(),
            Expression::UNARYPREFIX(unary_prefix_expression) => todo!(),
            Expression::UNARYSUFFIX(unary_postfix_expression) => todo!(),
            Expression::BINARYEXPRESSION(binary_expression) => panic!("can't get address of binary expression?"),
            Expression::TERNARYEXPRESSION(ternary_expr) => panic!("can't get address of ternary expression"),
            Expression::CAST(cast_expression) => panic!("can't get address of a cast"),
            Expression::SIZEOF(sizeof_expr) => panic!("can't get address of sizeof expression"),
        }
    }
}

/**
 * tries to parse the tokens queue starting at previous_queue_idx, to find an expression
 * returns an expression(entirely consumed), else none
 */
pub fn try_consume_whole_expr(tokens_queue: &TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<Expression> {
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
    if let Some(x) = ArrayInitialisation::try_consume_whole_expr(tokens_queue, previous_queue_idx, scope_data) {
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
                        if let Some(index_expr) = try_parse_array_index(tokens_queue, &curr_queue_idx, scope_data) {
                            return Some(Expression::UNARYPREFIX(index_expr));//since a[b] = *(a+b), indexing returns a unary prefix
                        }

                        if let Some(func) = FunctionCall::try_consume_whole_expr(tokens_queue, &curr_queue_idx, scope_data) {
                            return Some(Expression::FUNCCALL(func));
                        }

                        if let Some(access) = try_parse_member_access(tokens_queue, &curr_queue_idx, scope_data) {
                            return Some(Expression::STRUCTMEMBERACCESS(access));
                        }
                    }

                    let last_token = tokens_queue.peek_back(&curr_queue_idx, &scope_data).unwrap();
                    let ends_with_valid_suffix = last_token
                        .as_punctuator()
                        .and_then(|punc| punc.as_unary_suffix_precendece())
                        .is_some_and(|precedence| precedence == precedence_required);

                    if ends_with_valid_suffix {
                        if let Some(x) = try_parse_unary_suffix(tokens_queue, &curr_queue_idx, scope_data) {
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
                        if let Some(x) = try_parse_unary_prefix(tokens_queue, &curr_queue_idx, scope_data) {
                            return Some(Expression::UNARYPREFIX(x));
                        }
                    }

                    if precedence_required == 2 {
                        //parse cast expression
                        if let Some(cast) = try_parse_cast(tokens_queue, &curr_queue_idx, scope_data) {
                            return Some(Expression::CAST(cast));
                        }
                        if let Some(sizeof_expr) = try_parse_sizeof(tokens_queue, &curr_queue_idx, scope_data) {
                            return Some(Expression::SIZEOF(sizeof_expr));
                        }
                    }

                    if precedence_required == 13 {
                        //parse ternary conditional
                        if let Some(ternary) = try_parse_ternary(tokens_queue, &curr_queue_idx, scope_data) {
                            return Some(Expression::TERNARYEXPRESSION(ternary));
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
                    skip_in_squiggly_brackets: false,
                    skip_in_ternary_true_branch: false
                };

                let operator_indexes = tokens_queue.split_by_closure_matches(&curr_queue_idx, associative_direction, operator_matching_closure, &exclusions);

                for operator_idx in operator_indexes {
                    //try to find an operator
                    //note that the operator_idx is a slice of just the operator

                    match try_parse_binary_expr(tokens_queue, &curr_queue_idx, operator_idx, scope_data) {
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

pub fn promote(location: StackItemKey, original: DataType, promoted_type: DataType, stack_data: &mut SimpleStackFrame, struct_info: &dyn GetStructUnion) -> (IROperation, StackItemKey) {
    let result = stack_data.allocate(promoted_type.memory_size(struct_info));
    let op = match (original, promoted_type) {
        (DataType::RAW(BaseType::Scalar(from_type)), DataType::RAW(BaseType::Scalar(to_type))) =>
            IROperation::CAST { from: Storage::Stack(location), from_type, to: Storage::Stack(result), to_type },

        _ => todo!()
    };

    (op, result)
}

/// Returns assembly to handle the assignment and a copy of the data assigned (rhs value promoted to lhs type)
pub fn generate_assembly_for_assignment(lhs: &Expression, rhs: &Expression, asm_data: &AsmData, stack_data: &mut SimpleStackFrame, global_asm_data: &GlobalAsmData) -> (IRCode, Option<StackItemKey>) {
    let mut result = IRCode::make_empty();

    let promoted_type = lhs.get_type(asm_data);//TODO inline into match statement

    match (&promoted_type, rhs) {
        //initialising array to string literal
        (DataType::ARRAY {..}, Expression::STRINGLITERAL(string_init)) => {
            result.merge(&assembly_for_array_assignment(
                lhs,
                string_init.zero_fill_and_flatten_to_iter(&promoted_type),
                &DataType::RAW(BaseType::Scalar(ScalarType::Integer(IntegerType::I8))),
                asm_data, stack_data, global_asm_data
            ));

            (result, None)
        },

        //initialising array to array literal
        (DataType::ARRAY { .. }, Expression::ARRAYLITERAL(array_init)) => {
            //convert int x[2][2] to int x[4] for easy assigning of values
            unwrap_let!(DataType::ARRAY { element: array_element_type, .. } = promoted_type.flatten_nested_array());
            result.merge(&assembly_for_array_assignment(lhs, array_init.zero_fill_and_flatten_to_iter(&promoted_type), array_element_type.as_ref(), asm_data, stack_data, global_asm_data));

            (result, None)
        },

        (DataType::ARRAY { .. }, x) => panic!("tried to set {:?} to {:?}", lhs, x),

        (data_type, _) => {
            //maybe more special cases for struct assignment etc?

            //put address of lvalue on stack
            let (lhs_asm, lhs_addr_ptr) = lhs.get_address(asm_data, stack_data, global_asm_data);
            result.merge(&lhs_asm);
            
            //put the value to assign in acc, and cast to correct type
            let (rhs_asm, rhs_value) = rhs.generate_ir(asm_data, stack_data, global_asm_data);
            let (rhs_cast_asm, rhs_casted_value) = promote(rhs_value.unwrap(), rhs.get_type(asm_data), promoted_type.clone(), stack_data, asm_data);
            result.merge(&rhs_asm);
            result.add_instruction(rhs_cast_asm);

            result.add_comment("assigning to a stack variable");

            //save to memory
            result.add_instruction(IROperation::MOV {
                from: Storage::Stack(rhs_casted_value),
                to: Storage::IndirectAddress(lhs_addr_ptr),
                size: promoted_type.memory_size(asm_data),
            });

            (result, Some(rhs_casted_value))
        },
    }
}

/// Assigns `array_items` to `lhs`, where each item is of type `array_element_type`
fn assembly_for_array_assignment(lhs: &Expression,array_items: Vec<Expression>, array_element_type: &DataType, asm_data: &AsmData, stack_data: &mut SimpleStackFrame, global_asm_data: &GlobalAsmData) -> IRCode {
    let mut result = IRCode::make_empty();
    let array_element_size = array_element_type.memory_size(asm_data);

    //get address of destination array
    let (lhs_addr_asm, lhs_addr_ptr) = lhs.get_address(asm_data, stack_data, global_asm_data);
    result.merge(&lhs_addr_asm);

    //this stores the address of the to-be initialised element
    let lhs_current = stack_data.allocate(PTR_SIZE);
    result.add_instruction(IROperation::MOV {
        from: Storage::Stack(lhs_addr_ptr),
        to: Storage::Stack(lhs_current),
        size: PTR_SIZE,
    });

    //this generates the following c-style code to assign the array literal to the destination array
    //for 2d arrays, this code reinteprets it as a 1d array, using zero_fill_and_flatten_to_iter which flattens to 1d array
    //T* lhs_current = array;
    //for(int i=0;i<array_size;i++){
    //  T item_value = array_literal[i]
    //  *lhs_current = item_value;
    //  lhs_current++;
    //}
    for (i, item) in array_items.iter().enumerate() {

        //generate the item and store in `item_value`
        let (item_assignment, item_value) = item.generate_ir(asm_data, stack_data, global_asm_data);
        result.merge(&item_assignment);

        //place `item_value` in the next array index to be initialised
        result.add_commented_instruction(IROperation::MOV {
            from: Storage::Stack(item_value.unwrap()),
            to: Storage::IndirectAddress(lhs_current),
            size: array_element_size,
        }, format!("initialising element {} of array", i));

        //increment lhs_current
        result.add_instruction(IROperation::ADD {
            lhs: Storage::Stack(lhs_current),
            rhs: Storage::Constant(array_element_size.as_imm()),
            to: Storage::Stack(lhs_current),
            data_type: ScalarType::Integer(IntegerType::U64),
        });
    }

    result
}

/**
 * tries to parse the expression as a unary prefix and the operand, for example ++x or *(x->foo)
 * if the parse was successful, an expression is returned
 * else, you get None
 */
fn try_parse_unary_prefix(tokens_queue: &TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<UnaryPrefixExpression> {
    let mut curr_queue_idx = previous_queue_idx.clone();
    
    let unary_op: UnaryPrefixOperator = tokens_queue.consume(&mut curr_queue_idx, &scope_data)
    .and_then(|tok| tok.as_punctuator())
    .and_then(|punc| punc.try_into().ok())?;//get unary operator, or return if it isn't one

    let operand = try_consume_whole_expr(tokens_queue, &curr_queue_idx, scope_data)?;

    Some(UnaryPrefixExpression::new(unary_op, operand))
}

fn try_parse_unary_suffix(tokens_queue: &TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<UnaryPostfixExpression> {
    let mut curr_queue_idx = previous_queue_idx.clone();
    
    let unary_op: UnaryPostfixOperator = tokens_queue.peek_back(&curr_queue_idx, &scope_data)
    .and_then(|tok| tok.as_punctuator())
    .and_then(|punc| punc.try_into().ok())?;//get unary operator, or return if it isn't one

    curr_queue_idx.max_index -= 1;//consume the last token

    let operand = try_consume_whole_expr(tokens_queue, &curr_queue_idx, scope_data)?;

    Some(UnaryPostfixExpression::new(unary_op, operand))
}

/**
 * tries to parse the left and right hand side of operator_idx, as a binary expression e.g 1 + 2 split by "+"
 * if this parse was successful, an expression is returned
 * else, you get None
 */
fn try_parse_binary_expr(tokens_queue: &TokenQueue, curr_queue_idx: &TokenQueueSlice, operator_idx: usize, scope_data: &mut ParseData) -> Option<BinaryExpression> {
    //split to before and after the operator
    let (left_part, right_part) = tokens_queue.split_at(operator_idx, curr_queue_idx);

    //try and parse the left and right hand sides, propogating errors
    let parsed_left = try_consume_whole_expr(tokens_queue, &left_part, scope_data)?;
    let parsed_right = try_consume_whole_expr(tokens_queue, &right_part, scope_data)?;

    let operator = tokens_queue.peek(&TokenQueueSlice { index: operator_idx, max_index: operator_idx+1 }, &scope_data)//get token in the middle
    .and_then(|x| x.as_punctuator())//try to convert to punctuator
    .and_then(|x| BinaryExpressionOperator::from_punctuator(x))?;//try to convert to binary expression operator

    Some(BinaryExpression::new(parsed_left, operator, parsed_right))
}

fn try_parse_array_index(tokens_queue: &TokenQueue, curr_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<UnaryPrefixExpression> {
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

        let index_expr = try_consume_whole_expr(tokens_queue, &index_slice, scope_data)?;
        let array_expr = try_consume_whole_expr(tokens_queue, &array_slice, scope_data)?;

        //a[b] == *(a+b) in C
        return Some(
            UnaryPrefixExpression::new(UnaryPrefixOperator::Dereference, //dereference
                Expression::BINARYEXPRESSION(BinaryExpression::new(array_expr, BinaryExpressionOperator::Add, index_expr))//pointer plus index
            )
        );
    }

    None
}

fn try_parse_member_access(tokens_queue: &TokenQueue, expr_slice: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<MemberAccess> {

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
        let struct_tree = try_consume_whole_expr(tokens_queue, &curr_queue_idx, scope_data)?;

        return Some(MemberAccess::new(struct_tree, member_name));
    }
    
    None//failed to find correct identifiers
}

fn try_parse_sizeof(tokens_queue: &TokenQueue, expr_slice: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<SizeofExpr> {
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
    let base_expr = try_consume_whole_expr(tokens_queue, &curr_queue_idx, scope_data);
    //try and look for a data type
    let data_type = consume_fully_qualified_type(tokens_queue, &curr_queue_idx, scope_data)
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

fn try_parse_cast(tokens_queue: &TokenQueue, expr_slice: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<CastExpression> {
    let mut curr_queue_idx = expr_slice.clone();

    if tokens_queue.consume(&mut curr_queue_idx, scope_data)? != Token::PUNCTUATOR(Punctuator::OPENCURLY) {
        return None;//cast must start with "("
    }

    //match closure, as no nested brackets allowed in cast type
    let close_curly_idx = tokens_queue.find_closure_matches(&curr_queue_idx, false, |x| *x == Token::PUNCTUATOR(Punctuator::CLOSECURLY), &TokenSearchType::skip_nothing())?;

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
    let ASTMetadata { remaining_slice, resultant_tree: (new_type, _) } = consume_fully_qualified_type(tokens_queue, &new_type_slice, scope_data)?;
    assert!(remaining_slice.get_slice_size() == 0);//cannot be any remaining tokens in the cast type

    let base_expr = try_consume_whole_expr(tokens_queue, &remaining_expr_slice, scope_data)?;

    Some(CastExpression::new(new_type, base_expr))
}

fn try_parse_ternary(tokens_queue: &TokenQueue, expr_slice: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<TernaryExpr> {
    let mut curr_queue_idx = expr_slice.clone();

    let question_idx = tokens_queue.find_closure_matches(&curr_queue_idx, false, |x| *x == Token::PUNCTUATOR(Punctuator::QuestionMark), &TokenSearchType::skip_nothing())?;
    let condition = TokenQueueSlice { index: curr_queue_idx.index, max_index: question_idx };
    curr_queue_idx.index = question_idx + 1;//skip over the condition

    //skip EVERYTHING including the true branch in case of nested ternary expressions
    let skip_all = TokenSearchType { skip_in_curly_brackets: true, skip_in_square_brackets: true, skip_in_squiggly_brackets: true, skip_in_ternary_true_branch: true };
    let colon_idx = tokens_queue.find_closure_matches(&curr_queue_idx, false, |x| *x == Token::PUNCTUATOR(Punctuator::COLON), &skip_all).unwrap();

    let true_branch = TokenQueueSlice { index: curr_queue_idx.index, max_index: colon_idx };
    let false_branch = TokenQueueSlice {index: colon_idx+1, max_index: curr_queue_idx.max_index};

    Some(TernaryExpr::new(
        try_consume_whole_expr(tokens_queue, &condition, scope_data).unwrap(),
        try_consume_whole_expr(tokens_queue, &true_branch, scope_data).unwrap(),
        try_consume_whole_expr(tokens_queue, &false_branch, scope_data).unwrap()
    ))
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
            Expression::TERNARYEXPRESSION(ternary) => ternary.display_ast(f),
        }
    }
}