use crate::{ast_metadata::ASTMetadata, data_type::data_type::DataType, declaration::{consume_base_type, try_consume_declaration_modifiers, Declaration}, lexer::{keywords::Keyword, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, memory_size::MemoryLayout, parse_data::ParseData};

#[derive(Clone, Debug, PartialEq)]
pub struct StructDefinition {
    name: Option<String>,
    ordered_members: Option<Vec<(Declaration, MemoryLayout)>>,//decl and offset from start that this member is located
    size: Option<MemoryLayout>
}

impl StructDefinition {
    pub fn get_name(&self) -> &Option<String> {
        &self.name
    }

    pub fn calculate_size(&self) -> Option<MemoryLayout> {
        self.size
    }
    
    pub fn try_consume_struct_as_type(tokens_queue: &TokenQueue, curr_queue_idx: &mut TokenQueueSlice, scope_data: &mut ParseData) -> Option<StructDefinition> {
        if tokens_queue.consume(curr_queue_idx, &scope_data)? != Token::KEYWORD(Keyword::STRUCT) {
            return None;//needs preceding "struct"
        }
    
        let struct_name = if let Token::IDENTIFIER(x) = tokens_queue.consume(curr_queue_idx, &scope_data).unwrap() {x} else {todo!("found struct keyword, then non-identifier token. perhaps you tried to declare an anonymous struct inline?")};

        match tokens_queue.peek(curr_queue_idx, &scope_data).unwrap() {
            Token::PUNCTUATOR(Punctuator::OPENSQUIGGLY) => {
                let close_squiggly_idx = tokens_queue.find_matching_close_bracket(curr_queue_idx.index);
                let mut inside_variants = TokenQueueSlice{index:curr_queue_idx.index+1, max_index: close_squiggly_idx};//+1 to skip the {
                let remaining_slice = TokenQueueSlice{index:close_squiggly_idx, max_index:curr_queue_idx.max_index};

                let mut members = Vec::new();
                while let Some(new_member) = try_consume_struct_member(tokens_queue, &mut inside_variants, scope_data) {
                    members.push(new_member);
                }
                assert!(inside_variants.get_slice_size() == 0);//must consume all tokens in variants
                curr_queue_idx.index = remaining_slice.index;//update start index to be after the struct

                let (aligned_members, struct_size) = pad_members(members);//pad each member correctly

                let struct_definition = StructDefinition { name: Some(struct_name), ordered_members: Some(aligned_members), size: Some(struct_size) };
                scope_data.structs.add_struct(&struct_definition);

                Some(struct_definition)
            },

            _ => Some(scope_data.structs.get_struct(&struct_name).unwrap().clone())//TODO this could declare a struct?
        }
    }
}

fn try_consume_struct_member(tokens_queue: &TokenQueue, curr_queue_idx: &mut TokenQueueSlice, scope_data: &mut ParseData) -> Option<Declaration> {
    if curr_queue_idx.get_slice_size() == 0 {
        return None;
    }

    //consume the base type
    let base_type = consume_base_type(tokens_queue, curr_queue_idx, scope_data).unwrap();

    let semicolon_idx = tokens_queue.find_closure_matches(&curr_queue_idx, false, |x| *x == Token::PUNCTUATOR(Punctuator::SEMICOLON), &TokenSearchType::skip_nothing()).unwrap();

    let all_declarators_segment = TokenQueueSlice{index:curr_queue_idx.index, max_index:semicolon_idx.index};

    //consume pointer or array info, and member name
    let ASTMetadata{resultant_tree: Declaration { data_type: modifiers, name: member_name }, ..} = try_consume_declaration_modifiers(tokens_queue, &all_declarators_segment, &base_type, scope_data)?;

    let data_type = DataType::new_from_base_type(&base_type, modifiers.get_modifiers());

    curr_queue_idx.index = semicolon_idx.index + 1;

    Some(Declaration { data_type, name: member_name })
}

/**
 * returns padded members, and the overall size of the struct
 */
fn pad_members(members: Vec<Declaration>) -> (Vec<(Declaration, MemoryLayout)>, MemoryLayout) {
    let mut current_offset = MemoryLayout::new();

    let mut result = Vec::new();

    for m in &members {
        let alignment_bytes = calculate_alignment(m.get_type()).size_bytes();

        let bytes_past_last_boundary = current_offset.size_bytes() % alignment_bytes;
        let extra_padding = (alignment_bytes - bytes_past_last_boundary) % alignment_bytes;
        current_offset += MemoryLayout::from_bytes(extra_padding);//increase offset in this struct to reach optimal alignment

        result.push((m.clone(), current_offset));
        current_offset += m.get_type().memory_size();//increase offset in struct by the size of the member
    }

    //lastly, align to largest member's alignment, so that if this struct is in an array, subsequent structs are aligned
    let largest_member = members.iter()
        .map(|x| calculate_alignment(x.get_type()))
        .fold(MemoryLayout::new(), |acc, x| MemoryLayout::biggest(&acc, &x))
        .size_bytes();
    let bytes_past_last_boundary = current_offset.size_bytes() % largest_member;
    let extra_padding = (largest_member - bytes_past_last_boundary) % largest_member;
    current_offset += MemoryLayout::from_bytes(extra_padding);

    (result, current_offset)
}

fn calculate_alignment(data_type: &DataType) -> MemoryLayout {
    if data_type.is_array() {
        calculate_alignment(&data_type.remove_outer_modifier()) //array of x should align to a boundary of sizeof x, but call myself recursively to handle 2d arrays
    } else {
        data_type.memory_size()
    }
}

#[derive(Clone, Debug)]
pub struct StructList {
    struct_decls: Vec<StructDefinition>
}
impl StructList {
    pub fn new() -> StructList {
        StructList { struct_decls: Vec::new() }
    }
    pub fn add_struct(&mut self, new_definition: &StructDefinition) {
        if let Some(definition) = self.struct_decls.iter_mut().find(|x| x.name == new_definition.name){
            match (&definition.ordered_members, &new_definition.ordered_members) {
                (Some(_), Some(_)) => panic!("redefinition of struct {}", definition.name.clone().unwrap()),

                (None, Some(_)) => definition.ordered_members = new_definition.ordered_members.clone(),//new definition contains more data

                _ => {}//new definition provides no new data
            }
        } else {
            self.struct_decls.push(new_definition.clone());//add new struct
        }
    }

    pub fn get_struct(&self, name: &str) -> Option<&StructDefinition> {
        self.struct_decls.iter().find(|x| x.name == Some(name.to_string()))
    }
}