use crate::{data_type::base_type::BaseType, lexer::{precedence, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, number_literal::{LiteralValue, NumberLiteral}, parse_data::ParseData, string_literal::StringLiteral};

pub enum ConstexprValue {
    NUMBER(NumberLiteral),
    STRING(StringLiteral),
    POINTER(String),//string: label of thing I am pointing to
}

impl ConstexprValue {
    /**
     * folds a constant expression to a number
     */
    pub fn try_consume_whole_constexpr(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<ConstexprValue> {
        if previous_queue_idx.get_slice_size() == 1 {
            return match tokens_queue.peek(previous_queue_idx, scope_data).unwrap() {
                Token::NUMBER(number) => Some(ConstexprValue::NUMBER(number)),
                Token::STRING(str) => Some(ConstexprValue::STRING(str)),
                _ => panic!("found invalid token when consuming constant expression")
            };
        }

        let curr_queue_idx = previous_queue_idx.clone();

        for precedence_required in (precedence::min_precedence()..=precedence::max_precedence()).rev() {
            //find which direction the operators should be considered
            //true is l->r, which means that if true, scan direction for splitting points should be reversed
            let associative_direction = precedence::get_associativity_direction(precedence_required);

            if associative_direction {
                //look for unary postfix
                assert!(curr_queue_idx.max_index <= tokens_queue.tokens.len());

                if precedence_required == 1 {
                    todo!("const expr array indexing");
                    /*match try_parse_array_index(tokens_queue, &curr_queue_idx, scope_data) {
                        Some(x) => {return x},
                        None => {}
                    }*/
                }

            } else {
                //look for unary prefix as association is right to left
                let first_token = tokens_queue.peek(&curr_queue_idx, &scope_data).unwrap();

                let starts_with_valid_prefix = first_token
                    .as_punctuator()
                    .and_then(|punc| punc.as_unary_prefix_precedence())
                    .is_some_and(|precedence| precedence == precedence_required);

                if starts_with_valid_prefix {
                    match try_parse_constexpr_unary_prefix(tokens_queue, &curr_queue_idx, scope_data) {
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
                skip_in_squiggly_brackets: false
            };

            let operator_indexes = tokens_queue.split_by_closure_matches(&curr_queue_idx, associative_direction, operator_matching_closure, &exclusions);

            for operator_idx in operator_indexes {
                //try to find an operator
                //note that the operator_idx is a slice of just the operator

                match try_parse_binary_constexpr(tokens_queue, &curr_queue_idx, &operator_idx, scope_data) {
                    Some(x) => {return Some(x);}
                    None => {
                        continue;
                    }
                }

            }
        }
        panic!("could not parse constant expression");
    }
}

fn try_parse_constexpr_unary_prefix(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<ConstexprValue> {
    let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);
    
    let punctuator = tokens_queue.consume(&mut curr_queue_idx, &scope_data)?.as_punctuator()?;//get punctuator

    punctuator.as_unary_prefix_precedence()?;//ensure the punctuator is a valid unary prefix

    match punctuator {
        Punctuator::AMPERSAND => {
            if let Token::IDENTIFIER(label) = tokens_queue.consume(&mut curr_queue_idx, scope_data)? {
                Some(ConstexprValue::POINTER(label))//pointer to the label i.e &x is POINTER("x")
            } else {None}
        },
        _ => panic!("invalid unary prefix in constant expression")
    }
}

fn try_parse_binary_constexpr(tokens_queue: &mut TokenQueue, curr_queue_idx: &TokenQueueSlice, operator_idx: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<ConstexprValue> {
    //split to before and after the operator
    let (left_part, right_part) = tokens_queue.split_to_slices(operator_idx.index, curr_queue_idx);

    //try and parse the left and right hand sides, propogating errors
    let parsed_left = ConstexprValue::try_consume_whole_constexpr(tokens_queue, &left_part, scope_data)?;
    let parsed_right = ConstexprValue::try_consume_whole_constexpr(tokens_queue, &right_part, scope_data)?;

    let operator = tokens_queue.peek(&operator_idx, &scope_data).expect("couldn't peek")
        .as_punctuator().expect("couldn't cast to punctuator");

    match (parsed_left, parsed_right) {
        (ConstexprValue::NUMBER(x), ConstexprValue::NUMBER(y)) => {

            let lhs_val = x.get_value().clone();
            let rhs_val = y.get_value().clone();

            let new_value = match &operator {
                Punctuator::PLUS => {
                    match (lhs_val, rhs_val) {
                        (LiteralValue::SIGNED(l), LiteralValue::SIGNED(r)) => LiteralValue::SIGNED(l+r),
                        (LiteralValue::UNSIGNED(l), LiteralValue::UNSIGNED(r)) => LiteralValue::UNSIGNED(l+r),
                        _ => panic!("tried to add mixed signed-unsigned numbers in const expr")
                    }
                },
                Punctuator::DASH => {
                    match (lhs_val, rhs_val) {
                        (LiteralValue::SIGNED(l), LiteralValue::SIGNED(r)) => LiteralValue::SIGNED(l-r),
                        (LiteralValue::UNSIGNED(l), LiteralValue::UNSIGNED(r)) => LiteralValue::UNSIGNED(l-r),
                        _ => panic!("tried to subtract mixed signed-unsigned numbers in const expr")
                    }
                }
                _ => todo!()
            };

            //construct a number from the promoted type and the calculated value
            Some(ConstexprValue::NUMBER(NumberLiteral::new_from_literal_value(new_value).cast(&BaseType::I64)))//TODO proper base types!
        }
        _ => todo!()
    }
}