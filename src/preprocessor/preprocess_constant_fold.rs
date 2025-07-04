use unwrap_let::unwrap_let;

use crate::{ast_metadata::ASTMetadata, compilation_state::label_generator::LabelGenerator, constexpr_parsing::ConstexprValue, expression::expression::{try_consume_whole_expr, Expression}, lexer::{keywords::Keyword, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, number_literal::{literal_value::LiteralValue, typed_value::NumberLiteral}, parse_data::ParseData, preprocessor::preprocess_context::PreprocessContext};

/// Folds a constant for #if statements
pub fn fold(tokens: Vec<Token>, ctx: &PreprocessContext) -> ConstexprValue {
    println!("before defined: {:?}", tokens);
    let tokens = fix_defined(tokens, ctx);
    println!("after defined: {:?}", tokens);
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
            panic!("found string when parsing constant"),
        crate::constexpr_parsing::ConstexprValue::POINTER { label, offset } =>
            panic!("found a pointer when parsing constant"),
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