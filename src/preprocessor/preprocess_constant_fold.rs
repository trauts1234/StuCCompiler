use unwrap_let::unwrap_let;

use crate::{compilation_state::label_generator::LabelGenerator, constexpr_parsing::ConstexprValue, data_type::base_type::IntegerType, expression::expression::try_consume_whole_expr, lexer::{keywords::Keyword, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, number_literal::{ typed_value::NumberLiteral}, parse_data::ParseData, preprocessor::preprocess_context::PreprocessContext};

/// Folds a constant for #if statements
pub fn fold(tokens: Vec<Token>, ctx: &PreprocessContext) -> ConstexprValue {
    //replace defined(x) with 1 or 0
    let tokens = fix_defined(tokens, ctx);
    //replace the remaining macros
    let tokens = sub_definitions(tokens, ctx, &Vec::new());
    let tokens = TokenQueue::new(tokens);
    let slice = TokenQueueSlice::new();

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
pub fn sub_definitions(mut tokens: Vec<Token>, ctx: &PreprocessContext, excluded_ident: &Vec<String>) -> Vec<Token> {
    let mut i = 0;
    loop {
        if i >= tokens.len() { break }//reached end of tokens
        match tokens[i].clone() {

            Token::IDENTIFIER(macro_name) if is_replacable_macro(&macro_name, ctx, excluded_ident).is_some() => {
                //simple macro
                let definition = ctx.get_definition(&macro_name).unwrap();//get replacement
                let mut definition_exclusions =  excluded_ident.clone();
                definition_exclusions.push(macro_name);
                let definition = sub_definitions(definition, ctx, &definition_exclusions);//recursively substitute the replacement
                let definition_length = definition.len();
                tokens.splice(i..=i, definition);//replace the macro name with the definition
                i += definition_length;//skip over it as it has already had definitions substituted
            }


            _ => {i+=1}//skip over this token
        }
    }

    tokens
}

fn is_replacable_macro(ident: &String, ctx: &PreprocessContext, excluded_identifiers: &Vec<String>) -> Option<Vec<Token>> {
    let definition = ctx.get_definition(ident)?;
    if excluded_identifiers.contains(ident) {
        return None;//excluded
    }
    Some(definition)//identifier maps to a macro
}