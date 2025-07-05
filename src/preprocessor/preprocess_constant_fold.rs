use unwrap_let::unwrap_let;

use crate::{compilation_state::label_generator::LabelGenerator, constexpr_parsing::ConstexprValue, expression::expression::try_consume_whole_expr, lexer::{keywords::Keyword, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, number_literal::{literal_value::LiteralValue, typed_value::NumberLiteral}, parse_data::ParseData, preprocessor::preprocess_context::PreprocessContext};

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
            number_literal != NumberLiteral::new_from_literal_value(LiteralValue::INTEGER(0))
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
            NumberLiteral::new_from_literal_value(
                LiteralValue::INTEGER(if ctx.is_defined(&macro_name) {1} else {0})
            )
        ));

        fix_defined(tokens, ctx)//recursively handle any others
    } else {
        tokens//nothing to replace
    }
}

/// Substitutes definitions for macros, except ones with the name `excluded_ident`
pub fn sub_definitions(mut tokens: Vec<Token>, ctx: &PreprocessContext, excluded_ident: &Vec<String>) -> Vec<Token> {
    if let Some((i, macro_name, definition)) = tokens.iter()
        .enumerate()
        .filter_map(|(i, tok)| 
            if let Some((macro_name, macro_definition)) = is_replacable_macro(tok, ctx, excluded_ident) {
                Some((i, macro_name, macro_definition))
            } else {None}
        )
        .map(|(i, m, d)| (i, m.clone(), d))
        .next()
    {
        //recursively handle the definition
        let mut definition_exclusions = excluded_ident.clone();
        definition_exclusions.push(macro_name.clone());
        let definition = sub_definitions(definition, ctx, &definition_exclusions);

        let after = tokens.split_off(i+1);//get code after the match
        assert_eq!(tokens.pop(), Some(Token::IDENTIFIER(macro_name)));//pop the macro name
        tokens.extend(definition);//push the definition
        tokens.extend(sub_definitions(after, ctx, excluded_ident));//preprocess the remainder
    }

    tokens
}

fn is_replacable_macro(token: &Token, ctx: &PreprocessContext, excluded_identifiers: &Vec<String>) -> Option<(String, Vec<Token>)> {
    if let Token::IDENTIFIER(ident) = token {
        let definition = ctx.get_definition(ident)?;
        if excluded_identifiers.contains(ident) {
            return None;//excluded
        }
        Some((ident.to_string(), definition))//identifier maps to a macro
    } else {
        None//not and identifier
    }
}