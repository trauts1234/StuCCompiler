use crate::{lexer::{precedence, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, number_literal::NumberLiteral, scope_data::ScopeData, string_literal::StringLiteral};

pub enum ConstexprValue {
    NUMBER(NumberLiteral),
    STRING(StringLiteral),
    POINTER(String),//string: label of thing I am pointing to
}

impl ConstexprValue {
    /**
     * folds a constant expression to a number
     */
    pub fn try_consume_whole_constexpr(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ScopeData) -> Option<ConstexprValue> {
        if previous_queue_idx.get_slice_size() == 1 {
            return match tokens_queue.peek(previous_queue_idx, scope_data).unwrap() {
                Token::NUMBER(number) => Some(ConstexprValue::NUMBER(number)),
                Token::STRING(str) => Some(ConstexprValue::STRING(str)),
                _ => panic!("found invalid token when consuming constant expression")
            };
        }

        let mut curr_queue_idx = previous_queue_idx.clone();

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
            };

            let operator_indexes = tokens_queue.find_closure_matches(&curr_queue_idx, associative_direction, operator_matching_closure, &exclusions);

            for operator_idx in operator_indexes {
                //try to find an operator
                //note that the operator_idx is a slice of just the operator

                todo!("const expr binary expressions")

                /*match try_parse_binary_expr(tokens_queue, &curr_queue_idx, &operator_idx, scope_data) {
                    Some(x) => {return x;}
                    None => {
                        continue;
                    }
                }*/

            }
        }
        panic!("could not parse constant expression");
    }
}

fn try_parse_constexpr_unary_prefix(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ScopeData) -> Option<ConstexprValue> {
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