use crate::{ast_metadata::ASTMetadata, data_type::{recursive_data_type::DataType, storage_type::StorageDuration}, function_declaration::consume_fully_qualified_type, lexer::{keywords::Keyword, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, parse_data::ParseData};
use unwrap_let::unwrap_let;
pub struct Typedef;

impl Typedef {
    pub fn try_consume(tokens_queue: &TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<ASTMetadata<(String, DataType, StorageDuration)>> {

        let mut curr_queue_idx = previous_queue_idx.clone();

        //typedef must start with typedef keyword
        if tokens_queue.consume(&mut curr_queue_idx, scope_data)? != Token::KEYWORD(Keyword::TYPEDEF) {
            return None;
        }
        //after here do not return none, as it must be a typedef

        let semicolon_idx = tokens_queue.find_closure_matches(&curr_queue_idx, false, |x| *x == Token::PUNCTUATOR(Punctuator::SEMICOLON), &TokenSearchType::skip_all()).unwrap().index;

        //slice in which the data type is specified
        let type_slice = TokenQueueSlice {
            index: curr_queue_idx.index,
            max_index: semicolon_idx-1,//-1 to exclude the name being typedef'd to
        };
        //slice in which the name being associated with the type is located
        let name_slice = TokenQueueSlice {
            index: semicolon_idx-1,
            max_index: semicolon_idx,
        };
        let remaining = TokenQueueSlice {
            index: semicolon_idx+1,
            max_index: curr_queue_idx.max_index,
        };
        assert!(type_slice.get_slice_size() > 0);
        assert!(name_slice.get_slice_size() == 1);

        //get name
        unwrap_let!(Token::IDENTIFIER(name) = tokens_queue.peek(&name_slice, scope_data).unwrap());
        //get type, discarding storage duration
        let ASTMetadata { remaining_slice: type_remaining, resultant_tree: (type_represented, storage_duration) } = consume_fully_qualified_type(tokens_queue, &type_slice, scope_data).unwrap();
        assert!(type_remaining.get_slice_size() == 0);//must consume all of previous

        Some(ASTMetadata {
            remaining_slice: remaining,
            resultant_tree: (name, type_represented, storage_duration)
        })
    }
}