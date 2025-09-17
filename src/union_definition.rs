use std::fmt::Display;

use memory_size::MemorySize;
use uuid::Uuid;

use crate::{asm_gen_data::GetStructUnion, ast_metadata::ASTMetadata, declaration::Declaration, lexer::{keywords::Keyword, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, parse_data::ParseData, struct_definition::try_consume_member};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnionIdentifier {
    pub name: Option<String>,
    pub id: Uuid
}
impl Display for UnionIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "union {:?}.id{}", self.name, self.id)
    }
}

#[derive(Debug, Clone)]
pub struct UnionDefinition {
    pub ordered_members: Option<Vec<Declaration>>,
}

impl UnionDefinition {
    pub fn calculate_size(&self, struct_info: &dyn GetStructUnion) -> Option<MemorySize> {
        self.ordered_members
        .as_ref()
        .and_then(|members|
            members.iter()
            .map(|x| x.data_type.memory_size(struct_info))
            .max()
        )
    }

    pub fn get_member_data(&self, member_name: &str) -> Declaration {
        self.ordered_members.as_ref().expect("looking for member in struct with no members")
        .iter()
        .find(|decl| decl.name == member_name)//find correctly named member
        .expect(&format!("couldn't find struct member {}", member_name))
        .clone()
    }

    pub fn try_consume_union_as_type(tokens_queue: &TokenQueue, previous_slice: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<ASTMetadata<UnionIdentifier>> {
        let mut curr_queue_idx = previous_slice.clone();

        if tokens_queue.consume(&mut curr_queue_idx, &scope_data)? != Token::KEYWORD(Keyword::UNION) {
            return None;//needs preceding "union"
        }

        let union_name = if let Token::IDENTIFIER(x) = tokens_queue.peek(&mut curr_queue_idx, &scope_data).unwrap() {
            tokens_queue.consume(&mut curr_queue_idx, scope_data).unwrap();//consume the name
            Some(x)
        } else {
            None
        };

        match tokens_queue.peek(&curr_queue_idx, &scope_data) {
            Some(Token::PUNCTUATOR(Punctuator::OPENSQUIGGLY)) => {
                let close_squiggly_idx = tokens_queue.find_matching_close_bracket(curr_queue_idx.index);
                let mut inside_variants = TokenQueueSlice{index:curr_queue_idx.index+1, max_index: close_squiggly_idx};//+1 to skip the {
                let remaining_slice = TokenQueueSlice{index:close_squiggly_idx+1, max_index:curr_queue_idx.max_index};

                let mut members = Vec::new();
                while inside_variants.get_slice_size() > 0 {
                    let mut new_member = try_consume_member(tokens_queue, &mut inside_variants, scope_data);
                    members.append(&mut new_member);
                }

                assert!(inside_variants.get_slice_size() == 0);//must consume all tokens in variants

                let union_definition = UnionDefinition { ordered_members: Some(members),  };
                let union_identifier = scope_data.add_union(&union_name, &union_definition);

                Some(ASTMetadata {
                    remaining_slice,
                    resultant_tree: union_identifier
                })
            },

            _ => Some(ASTMetadata { 
                remaining_slice: curr_queue_idx,
                //add declaration and return identifier of it
                resultant_tree: scope_data.add_union(&union_name, &UnionDefinition { ordered_members: None })
            })
        }

    }
}