use crate::{asm_gen_data::GetStruct, ast_metadata::ASTMetadata, compilation_state::label_generator::LabelGenerator, data_type::{recursive_data_type::DataType, storage_type::StorageDuration}, debugging::DebugDisplay, declaration::Declaration, initialised_declaration::{consume_type_specifier, try_consume_declaration_modifiers}, lexer::{keywords::Keyword, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, parse_data::ParseData};
use memory_size::MemorySize;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct StructIdentifier {
    pub(crate) name: Option<String>,
    pub(crate) id: u32
}
impl DebugDisplay for StructIdentifier {
    fn display(&self) -> String {
        format!("{:?}.id{}", self.name, self.id)
    }
}

/**
 * before assembly generation, structs have not had padding calculated
 */
#[derive(Clone, Debug, PartialEq)]
pub struct UnpaddedStructDefinition {
    pub(crate) ordered_members: Option<Vec<Declaration>>
}

impl UnpaddedStructDefinition {
    /**
     * returns padded members, and the overall size of the struct
     */
    pub fn pad_members(&self, struct_info: &dyn GetStruct) -> StructDefinition {
        let mut current_offset = MemorySize::new();

        let mut result = Vec::new();
        if let Some(some_ordered_members) = self.ordered_members.as_ref() {
            for m in some_ordered_members {
                let alignment_bytes = calculate_alignment(m.get_type(), struct_info).size_bytes();
    
                let bytes_past_last_boundary = current_offset.size_bytes() % alignment_bytes;
                let extra_padding = (alignment_bytes - bytes_past_last_boundary) % alignment_bytes;
                current_offset += MemorySize::from_bytes(extra_padding);//increase offset in this struct to reach optimal alignment
    
                result.push((m.clone(), current_offset));
                current_offset += m.get_type().memory_size(struct_info);//increase offset in struct by the size of the member
            }
    
            //lastly, align to largest member's alignment, so that if this struct is in an array, subsequent structs are aligned
            let largest_member_alignment = self.ordered_members.as_ref().unwrap().iter()
                .map(|x| calculate_alignment(x.get_type(), struct_info))
                .fold(MemorySize::new(), |acc, x| acc.max(x))
                .size_bytes();
            let bytes_past_last_boundary = current_offset.size_bytes() % largest_member_alignment;
            let extra_padding = (largest_member_alignment - bytes_past_last_boundary) % largest_member_alignment;
            current_offset += MemorySize::from_bytes(extra_padding);
    
            StructDefinition { ordered_members: Some(result), size: Some(current_offset) }
        } else {
            StructDefinition {ordered_members: None, size: None }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StructDefinition {
    ordered_members: Option<Vec<(Declaration, MemorySize)>>,//decl and offset from start that this member is located
    size: Option<MemorySize>
}

impl StructDefinition {

    pub fn calculate_size(&self) -> Option<MemorySize> {
        self.size
    }

    pub fn get_member_data(&self, member_name: &str) -> (Declaration, MemorySize) {
        self.ordered_members.as_ref().expect("looking for member in struct with no members")
        .iter()
        .find(|(decl, _)| decl.name == member_name)//find correctly named member
        .expect(&format!("couldn't find struct member {}", member_name))
        .clone()
    }
    pub fn get_all_members(&self) -> &Option<Vec<(Declaration, MemorySize)>> {
        &self.ordered_members
    }
    
    pub fn try_consume_struct_as_type(tokens_queue: &TokenQueue, previous_slice: &TokenQueueSlice, scope_data: &mut ParseData, struct_label_gen: &mut LabelGenerator) -> Option<ASTMetadata<StructIdentifier>> {

        let mut curr_queue_idx = previous_slice.clone();

        if tokens_queue.consume(&mut curr_queue_idx, &scope_data)? != Token::KEYWORD(Keyword::STRUCT) {
            return None;//needs preceding "struct"
        }

        let struct_name = if let Token::IDENTIFIER(x) = tokens_queue.peek(&mut curr_queue_idx, &scope_data).unwrap() {
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
                    let mut new_member = try_consume_struct_member(tokens_queue, &mut inside_variants, scope_data, struct_label_gen);
                    members.append(&mut new_member);
                }

                assert!(inside_variants.get_slice_size() == 0);//must consume all tokens in variants

                let struct_definition = UnpaddedStructDefinition { ordered_members: Some(members),  };
                let struct_identifier = scope_data.add_struct(&struct_name, &struct_definition, struct_label_gen);

                Some(ASTMetadata {
                    remaining_slice,
                    resultant_tree: struct_identifier
                })
            },

            _ => Some(ASTMetadata { 
                remaining_slice: curr_queue_idx,
                //add declaration and return identifier of it
                resultant_tree: scope_data.add_struct(&struct_name, &UnpaddedStructDefinition { ordered_members: None }, struct_label_gen)
            })
        }
    }
}

///in struct definitions, this will consume the `int a,b;` part of `struct {int a,b;char c;}`
fn try_consume_struct_member(tokens_queue: &TokenQueue, curr_queue_idx: &mut TokenQueueSlice, scope_data: &mut ParseData, struct_label_gen: &mut LabelGenerator) -> Vec<Declaration> {

    //consume the base type
    let ASTMetadata { remaining_slice, resultant_tree: (base_type, storage_duration) } = consume_type_specifier(tokens_queue, &curr_queue_idx, scope_data, struct_label_gen).unwrap();
    assert_eq!(storage_duration, StorageDuration::Default);//cannot specify static or extern in a struct

    curr_queue_idx.index = remaining_slice.index;//consume it and let the calling function know

    let semicolon_idx = tokens_queue.find_closure_matches(&curr_queue_idx, false, |x| *x == Token::PUNCTUATOR(Punctuator::SEMICOLON), &TokenSearchType::skip_all()).unwrap();

    let all_declarators_segment = TokenQueueSlice{index:curr_queue_idx.index, max_index:semicolon_idx.index};

    let declarator_segments = tokens_queue.split_outside_parentheses(&all_declarators_segment, |x| *x == Token::PUNCTUATOR(Punctuator::COMMA), &TokenSearchType::skip_all());

    curr_queue_idx.index = semicolon_idx.index + 1;

    declarator_segments
    .iter()//go through each comma separated declaration
    .map(|declarator_segment| {
        try_consume_declaration_modifiers(tokens_queue, &declarator_segment, &base_type, scope_data)//convert it into a declaration
        .unwrap()
        .resultant_tree//extract the declaration
    })
    .collect()

}

fn calculate_alignment(data_type: &DataType, struct_info: &dyn GetStruct) -> MemorySize {
    if let DataType::ARRAY {..} = data_type {
        calculate_alignment(&data_type.remove_outer_modifier(), struct_info) //array of x should align to a boundary of sizeof x, but call myself recursively to handle 2d arrays
    } else {
        data_type.memory_size(struct_info)
    }
}