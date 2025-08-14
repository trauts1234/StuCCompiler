use std::collections::HashMap;

use unwrap_let::unwrap_let;

use crate::{compilation_state::label_generator::LabelGenerator, constexpr_parsing::ConstexprValue, data_type::base_type::IntegerType, expression::expression::try_consume_whole_expr, lexer::{keywords::Keyword, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, number_literal::typed_value::NumberLiteral, parse_data::ParseData, preprocessor::{preprocess_context::PreprocessContext, preprocess_token::MacroFunction}};

/// Folds a constant for #if statements
pub fn fold(tokens: Vec<Token>, ctx: &PreprocessContext) -> ConstexprValue {
    //replace defined(x) with 1 or 0
    let tokens = fix_defined(tokens, ctx);
    //replace the remaining macros
    let tokens = sub_definitions(tokens, ctx, &Vec::new(), &HashMap::new());
    let tokens = TokenQueue::new(tokens);
    let slice = TokenQueueSlice::new();

    println!("{:?}", tokens.tokens);

    let resultant_tree = try_consume_whole_expr(&tokens, &slice, &mut ParseData::make_empty(), &mut LabelGenerator::default()).unwrap();

    (&resultant_tree).try_into().unwrap()
}

/// Compares whether a #if would consider this constexpr value as true
pub fn is_true(folded: ConstexprValue) -> bool {
    match folded{
        crate::constexpr_parsing::ConstexprValue::NUMBER(number_literal) => {
            number_literal != NumberLiteral::INTEGER{data: 0, data_type: IntegerType::I32}
        }
        crate::constexpr_parsing::ConstexprValue::STRING(string_literal) =>
            panic!("found string when parsing constant: {:?}", string_literal),
        crate::constexpr_parsing::ConstexprValue::POINTER { label, offset } =>
            panic!("found a pointer when parsing constant: {:?} with offset {:?}", label, offset),
        crate::constexpr_parsing::ConstexprValue::ZEROES => false,
    }
}

fn fix_defined(mut tokens: Vec<Token>, ctx: &PreprocessContext) -> Vec<Token> {
    let defined_idx = tokens.iter().position(|tok| *tok == Token::KEYWORD(Keyword::DEFINED));

    if let Some(idx) = defined_idx {
        tokens.remove(idx);//remove defined keyword
        let macro_name = match tokens.remove(idx) {
            Token::IDENTIFIER(x) => x,// defined x
            Token::PUNCTUATOR(Punctuator::OPENCURLY) => {
                unwrap_let!(Token::IDENTIFIER(name) = tokens.remove(idx));
                assert_eq!(tokens.remove(idx), Token::PUNCTUATOR(Punctuator::CLOSECURLY));//remove the close bracket

                name
            }
            _ => panic!("invalid token after a defined keyword")
        };

        tokens.insert(idx, Token::NUMBER(
            NumberLiteral::INTEGER {
                data: if ctx.get_definition(&macro_name).is_some() || ctx.get_macro_func(&macro_name).is_some() {1} else {0},
                data_type: IntegerType::I32
            }
        ));

        fix_defined(tokens, ctx)//recursively handle any others
    } else {
        tokens//nothing to replace
    }
}

/// Substitutes definitions for macros, except ones with the name `excluded_ident`
/// 
/// TODO perhaps an `outer variables` param that contains substitutable parameters, to substitute macro function params
pub fn sub_definitions(tokens: Vec<Token>, ctx: &PreprocessContext, excluded_ident: &Vec<String>, substitutions: &HashMap<String, Vec<Token>>) -> Vec<Token> {
    let queue = TokenQueue::new(tokens);
    let mut rem = TokenQueueSlice::new();
    let mut result = Vec::new();
    loop {
        if queue.no_remaining_tokens(&rem) { break }//reached end of tokens
        match queue.consume(&mut rem, &ParseData::make_empty()).unwrap() {

            Token::IDENTIFIER(macro_name) if ctx.get_definition(&macro_name).is_some() && !excluded_ident.contains(&macro_name) => {
                //simple macro
                let definition = ctx.get_definition(&macro_name).unwrap();//get replacement
                let mut definition_exclusions =  excluded_ident.clone();
                definition_exclusions.push(macro_name);
                let definition = sub_definitions(definition, ctx, &definition_exclusions, substitutions);//recursively substitute the replacement
                result.extend(definition);//add the replacement
            }

            Token::IDENTIFIER(macro_name) if ctx.get_macro_func(&macro_name).is_some() && !excluded_ident.contains(&macro_name) => {
                //get the definition
                let MacroFunction {body, params} = ctx.get_macro_func(&macro_name).unwrap();
                //find the end of the param list
                let close_bracket = queue.find_matching_close_bracket(rem.index);
                //consume the "("
                assert_eq!(queue.consume(&mut rem, &ParseData::make_empty()).unwrap(), Token::PUNCTUATOR(Punctuator::OPENCURLY));

                //get the args
                let args_slice = TokenQueueSlice{index:rem.index, max_index: close_bracket};
                let args = queue.split_outside_parentheses(&args_slice, |x| *x == Token::PUNCTUATOR(Punctuator::COMMA), &TokenSearchType::skip_all_brackets());
                
                let mut param_substitutions = substitutions.clone();//start with existing substitutions
                for (param, arg) in params.into_iter().zip(args.into_iter()) {
                    //get the param and match it to the arg
                    println!("substituting {:?} for {:?}", param, &queue.tokens[arg.index..arg.max_index]);
                    param_substitutions.insert(param, queue.tokens[arg.index..arg.max_index].to_vec());
                }

                let mut definition_exclusions =  excluded_ident.clone();
                definition_exclusions.push(macro_name);

                let body = sub_definitions(body, ctx, &definition_exclusions, &param_substitutions);
                result.extend(body);

                //consume the close bracket
                rem.index = close_bracket;//skip the already-handled param list
                assert_eq!(queue.consume(&mut rem, &ParseData::make_empty()).unwrap(), Token::PUNCTUATOR(Punctuator::CLOSECURLY));
            }

            Token::IDENTIFIER(sub_name) if substitutions.contains_key(&sub_name) => {
                //TODO perhaps this has priority over macros?
                //substitute the definition
                result.extend(substitutions.get(&sub_name).unwrap().iter().cloned());
            }


            x => {result.push(x);}//normal token
        }
    }
 
    result
}