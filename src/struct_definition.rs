use crate::{ast_metadata::ASTMetadata, data_type::data_type::DataType, declaration::{consume_base_type, try_consume_declaration_modifiers, Declaration}, lexer::{keywords::Keyword, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, parse_data::ParseData};

#[derive(Clone, Debug)]
pub struct StructDefinition {
    name: Option<String>,
    ordered_members: Option<Vec<Declaration>>
}

impl StructDefinition {
    pub fn try_consume_struct_as_type(tokens_queue: &TokenQueue, curr_queue_idx: &mut TokenQueueSlice, scope_data: &mut ParseData) -> Option<StructDefinition> {
        if tokens_queue.consume(curr_queue_idx, &scope_data)? != Token::KEYWORD(Keyword::STRUCT) {
            return None;//needs preceding "struct"
        }
    
        let struct_name = if let Token::IDENTIFIER(x) = tokens_queue.consume(curr_queue_idx, &scope_data).unwrap() {x} else {todo!("found struct keyword, then non-identifier token. perhaps you tried to declare an anonymous struct inline?")};

        match tokens_queue.peek(curr_queue_idx, &scope_data).unwrap() {
            Token::PUNCTUATOR(Punctuator::OPENSQUIGGLY) => {
                let close_squiggly_idx = tokens_queue.find_matching_close_bracket(curr_queue_idx.index);
                let mut inside_variants = TokenQueueSlice{index:curr_queue_idx.index+1, max_index: close_squiggly_idx};//+1 to skip the {
                let mut remaining_slice = TokenQueueSlice{index:close_squiggly_idx, max_index:curr_queue_idx.max_index};

                if tokens_queue.consume(&mut remaining_slice, &scope_data)? == Token::PUNCTUATOR(Punctuator::SEMICOLON) {
                    //no trailinig semicolon
                    panic!("creating a variable of an enum inline with a definition not implemented");
                }

                let mut members = Vec::new();
                while let Some(new_member) = try_consume_struct_member(tokens_queue, &mut inside_variants, scope_data) {
                    members.push(new_member);
                }
                assert!(inside_variants.get_slice_size() == 0);//must consume all tokens in variants
                curr_queue_idx.index = remaining_slice.index;//update start index to be after the enum
                let struct_definition = StructDefinition { name: Some(struct_name), ordered_members: Some(members) };
                scope_data.structs.add_struct(&struct_definition);

                Some(struct_definition)
            },

            _ => Some(scope_data.structs.get_struct(&struct_name).unwrap().clone())
        }
    }
}

fn try_consume_struct_member(tokens_queue: &TokenQueue, curr_queue_idx: &mut TokenQueueSlice, scope_data: &mut ParseData) -> Option<Declaration> {
    if curr_queue_idx.get_slice_size() == 0 {
        return None;
    }

    //consume the base type
    let base_type = consume_base_type(tokens_queue, curr_queue_idx, scope_data).unwrap();

    let semicolon_idx = tokens_queue.find_closure_in_slice(&curr_queue_idx, false, |x| *x == Token::PUNCTUATOR(Punctuator::SEMICOLON)).unwrap();

    let all_declarators_segment = TokenQueueSlice{index:curr_queue_idx.index, max_index:semicolon_idx.index};

    //consume pointer or array info, and member name
    let ASTMetadata{resultant_tree: Declaration { data_type: modifiers, name: member_name }, ..} = try_consume_declaration_modifiers(tokens_queue, &all_declarators_segment, &base_type, scope_data)?;

    let data_type = DataType::new_from_base_type(&base_type, modifiers.get_modifiers());

    curr_queue_idx.index = semicolon_idx.index + 1;

    Some(Declaration { data_type, name: member_name })
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