use std::collections::HashMap;

use crate::{data_type::{base_type::BaseType, data_type::DataType}, lexer::{keywords::Keyword, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, number_literal::NumberLiteral, parse_data::ParseData};

/**
 * stores all the enums in a current scope
 */
#[derive(Clone, Debug)]
pub struct EnumList {
    all_variants: HashMap<String, NumberLiteral>,//converts enum variant name to number literal

    all_enum_names: HashMap<String, DataType>//converts enum name to the enum's type
}

/**
 * if a new enum is found, scope_data is updated and the data type of the enum is returned
 * 
 * this handles consuming enum definitions, which return a data type
 * think:
 * enum x {A,B} func() {return A;}
 * is legal code as the enum part returns integer type
 */
pub fn try_consume_enum_as_type(tokens_queue: &TokenQueue, curr_queue_idx: &mut TokenQueueSlice, scope_data: &mut ParseData) -> Option<DataType> {
    
    if tokens_queue.consume(curr_queue_idx, &scope_data)? != Token::KEYWORD(Keyword::ENUM) {
        return None;//needs preceding "enum"
    }

    let enum_name = if let Token::IDENTIFIER(x) = tokens_queue.consume(curr_queue_idx, &scope_data).unwrap() {x} else {todo!("found enum keyword, then non-identifier token. perhaps you tried to declare an anonymous enum inline?")};

    match tokens_queue.peek(curr_queue_idx, &scope_data).unwrap() {
        Token::PUNCTUATOR(Punctuator::OPENSQUIGGLY) => {
            let close_squiggly_idx = tokens_queue.find_matching_close_bracket(curr_queue_idx.index);
            let mut inside_variants = TokenQueueSlice{index:curr_queue_idx.index+1, max_index: close_squiggly_idx};//+1 to skip the {
            let mut remaining_slice = TokenQueueSlice{index:close_squiggly_idx, max_index:curr_queue_idx.max_index};

            if tokens_queue.consume(&mut remaining_slice, &scope_data)? == Token::PUNCTUATOR(Punctuator::SEMICOLON) {
                //no trailinig semicolon
                panic!("creating a variable of an enum inline with a definition not implemented");
            }

            let mut prev_num = -1;//this is a temporary counter, as when custom types are used for enums, this may break

            let data_type = DataType::new_from_base_type(&BaseType::I32, &Vec::new());

            while let Some(variant) = try_consume_enum_variant_definition(tokens_queue, &mut inside_variants, &mut prev_num, scope_data) {
                scope_data.enums.add_variant(variant);
            }
            assert!(inside_variants.get_slice_size() == 0);//must consume all tokens in variants
            curr_queue_idx.index = remaining_slice.index;//update start index to be after the enum
            scope_data.enums.add_enum(enum_name, data_type.clone());

            Some(data_type)
        }
        _ => {
            //enum usage, since there is no {variant_a, variant_b} part
            Some(scope_data.enums.get_enum_data_type(&enum_name).unwrap().clone())
        }
    }
}

impl EnumList {
    pub fn new() -> EnumList {
        EnumList { all_variants:HashMap::new(), all_enum_names: HashMap::new() }
    }
    pub fn add_variant(&mut self, variant: (String, NumberLiteral)) {
        let (var_name, var_num) = variant;
        
        if self.all_variants.contains_key(&var_name) {
            panic!("redefinition of enum variant: {}", var_name);
        }
        self.all_variants.insert(var_name, var_num);
    }
    pub fn add_enum(&mut self, name: String, data_type: DataType) {
        if self.all_enum_names.contains_key(&name) {
            panic!("tried to double define enum: {}", name);
        }
        self.all_enum_names.insert(name, data_type);
    }
    pub fn get_enum_data_type(&self, enum_name: &str) -> Option<&DataType> {
        self.all_enum_names.get(enum_name)
    }
    pub fn try_get_variant(&self, enum_variant: &str) -> Option<&NumberLiteral> {
        self.all_variants.get(enum_variant)
    }
}

/**
 * consumes tokens_queue by modifying remaining_tokens and returns an enum variant if found
 * returns the enum variant name and the number it equals
 */
fn try_consume_enum_variant_definition(tokens_queue: &TokenQueue, remaining_tokens: &mut TokenQueueSlice, prev_variant_number: &mut i32, scope_data: &mut ParseData) -> Option<(String, NumberLiteral)> {
    if remaining_tokens.get_slice_size() == 0 {
        return None;
    }

    if let Token::IDENTIFIER(variant_name) = tokens_queue.consume(remaining_tokens, &scope_data).unwrap() {
        if Some(Token::PUNCTUATOR(Punctuator::COMMA)) == tokens_queue.peek(&remaining_tokens, &scope_data) {
            tokens_queue.consume(remaining_tokens, &scope_data).unwrap();//found a comma after my definition, consume it
        }

        *prev_variant_number += 1;//later, when this is a NumberLiteral, you should call some sort of evaluate_const_expr(1 + prev_variant_number) as it could be different types, or worse

        return Some((variant_name, NumberLiteral::new(&prev_variant_number.to_string())))
    } else {
        panic!("tried to read enum variant but didn't find an identifier");
    }
}