use crate::{ast_metadata::ASTMetadata, data_type::data_type::DataType, lexer::{token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, number_literal::NumberLiteral};


struct EnumVariant {
    name: String,
    value: NumberLiteral
}

pub struct EnumDefinition {
    variants: Vec<EnumVariant>,
    data_type: DataType
}

impl EnumDefinition {
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice) -> Option<ASTMetadata<EnumDefinition>> {
        let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);
        todo!()
    }
}