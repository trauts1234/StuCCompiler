use std::collections::HashMap;

use crate::{data_type::{base_type::BaseType, data_type::DataType}, declaration::Declaration, lexer::{keywords::Keyword, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size::MemoryLayout, parse_data::ParseData};

#[derive(Clone, Debug)]
pub struct StructDefinition {
    struct_size: MemoryLayout,//TODO remember to include padding
    ordered_members: Vec<Declaration>
}

impl StructDefinition {
    pub fn as_data_type(&self) -> DataType {
        DataType::new_from_base_type(&BaseType::STRUCT(self.struct_size), &Vec::new())
    }
    pub fn try_consume_struct_as_type(tokens_queue: &TokenQueue, curr_queue_idx: &mut TokenQueueSlice, scope_data: &mut ParseData) -> Option<DataType> {
        if tokens_queue.consume(curr_queue_idx, &scope_data)? != Token::KEYWORD(Keyword::STRUCT) {
            return None;//needs preceding "struct"
        }
    
        let struct_name = if let Token::IDENTIFIER(x) = tokens_queue.consume(curr_queue_idx, &scope_data).unwrap() {x} else {todo!("found struct keyword, then non-identifier token. perhaps you tried to declare an anonymous struct inline?")};

        match tokens_queue.peek(curr_queue_idx, &scope_data).unwrap() {
            Token::PUNCTUATOR(Punctuator::OPENSQUIGGLY) => todo!(),

            _ => todo!()
        }
    }
}

#[derive(Clone, Debug)]
pub struct StructList {
    structs: HashMap<String, StructDefinition>
}
impl StructList {
    pub fn new() -> StructList {
        StructList { structs: HashMap::new() }
    }
    pub fn add_struct(&mut self, name: &str, definition: StructDefinition) {
        if self.structs.contains_key(name){
            panic!("double definition of struct {}", name);
        }

        self.structs.insert(name.to_string(), definition);
    }

    pub fn get_struct(&self, name: &str) -> Option<&StructDefinition> {
        self.structs.get(name)
    }
}