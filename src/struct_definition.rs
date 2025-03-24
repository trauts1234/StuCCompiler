use crate::{asm_gen_data::AsmData, asm_generation::{asm_comment, asm_line, LogicalRegister, RegisterName, PTR_SIZE}, ast_metadata::ASTMetadata, data_type::{base_type::BaseType, data_type::DataType}, declaration::{consume_base_type, try_consume_declaration_modifiers, Declaration}, expression::Expression, expression_visitors::{data_type_visitor::GetDataTypeVisitor, reference_assembly_visitor::ReferenceVisitor}, lexer::{keywords::Keyword, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, memory_size::MemoryLayout, parse_data::ParseData};
use std::collections::HashMap;
use std::fmt::Write;
use unwrap_let::unwrap_let;

#[derive(Clone, Debug, PartialEq)]
pub struct StructDefinition {
    name: Option<String>,
    ordered_members: Option<Vec<(Declaration, MemoryLayout)>>,//decl and offset from start that this member is located
    size: Option<MemoryLayout>
}

#[derive(Clone)]
pub struct StructMemberAccess {
    struct_tree: Box<Expression>,//need a tree or something to represent what gives me back the struct
    member_name: String,
}

impl StructMemberAccess {
    pub fn new(struct_tree: Expression, member_name: String) -> StructMemberAccess {
        StructMemberAccess { struct_tree: Box::new(struct_tree), member_name }
    }

    pub fn get_data_type(&self, asm_data: &AsmData) -> DataType {
        let struct_tree_type = self.struct_tree.accept(&mut GetDataTypeVisitor, asm_data);//get type of the tree that returns the struct

        assert!(struct_tree_type.is_bare_struct());//must be a struct

        unwrap_let!(BaseType::STRUCT(struct_type) = struct_tree_type.underlying_type());//get struct data

        let (member_decl, _) = struct_type.get_member_data(&self.member_name);//get the type of the member

        member_decl.get_type().clone()
    }

    pub fn put_addr_in_acc(&self, asm_data: &AsmData) -> String {
        let mut result = String::new();

        asm_comment!(result, "getting address of struct's member {}", self.member_name);
        //put tree's address in acc
        //add the member offset

        let ptr_reg = LogicalRegister::ACC.generate_reg_name(&PTR_SIZE);

        let struct_get_addr = self.struct_tree.accept(&mut ReferenceVisitor, asm_data);//assembly to get address of struct
        let struct_type = self.struct_tree.accept(&mut GetDataTypeVisitor, asm_data);//get data type of struct

        assert!(struct_type.is_bare_struct());
        unwrap_let!(BaseType::STRUCT(struct_definition) = struct_type.underlying_type());//get data from base type
        let (_, struct_member_offset) = struct_definition.get_member_data(&self.member_name);//get offset for the specific member

        asm_line!(result, "{}", struct_get_addr);//get address of struct
        asm_line!(result, "add {}, {}", ptr_reg, struct_member_offset.size_bytes());//go up by member offset

        result
    }
}

impl StructDefinition {
    pub fn get_name(&self) -> &Option<String> {
        &self.name
    }

    pub fn calculate_size(&self) -> Option<MemoryLayout> {
        self.size
    }

    pub fn get_member_data(&self, member_name: &str) -> (Declaration, MemoryLayout) {
        self.ordered_members
        .as_ref()
        .and_then(|members| members.iter().find(|(decl, _)| decl.name == member_name))//find correctly named member
        .cloned()
        .unwrap()
    }
    
    pub fn try_consume_struct_as_type(tokens_queue: &TokenQueue, previous_slice: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<ASTMetadata<StructDefinition>> {

        let mut curr_queue_idx = previous_slice.clone();

        if tokens_queue.consume(&mut curr_queue_idx, &scope_data)? != Token::KEYWORD(Keyword::STRUCT) {
            return None;//needs preceding "struct"
        }
    
        let struct_name = if let Token::IDENTIFIER(x) = tokens_queue.consume(&mut curr_queue_idx, &scope_data).unwrap() {x} else {todo!("found struct keyword, then non-identifier token. perhaps you tried to declare an anonymous struct inline?")};

        match tokens_queue.peek(&curr_queue_idx, &scope_data).unwrap() {
            Token::PUNCTUATOR(Punctuator::OPENSQUIGGLY) => {
                let close_squiggly_idx = tokens_queue.find_matching_close_bracket(curr_queue_idx.index);
                let mut inside_variants = TokenQueueSlice{index:curr_queue_idx.index+1, max_index: close_squiggly_idx};//+1 to skip the {
                let remaining_slice = TokenQueueSlice{index:close_squiggly_idx+1, max_index:curr_queue_idx.max_index};

                let mut members = Vec::new();
                while let Some(new_member) = try_consume_struct_member(tokens_queue, &mut inside_variants, scope_data) {
                    members.push(new_member);
                }
                assert!(inside_variants.get_slice_size() == 0);//must consume all tokens in variants

                let (aligned_members, struct_size) = pad_members(members);//pad each member correctly

                let struct_definition = StructDefinition { name: Some(struct_name), ordered_members: Some(aligned_members), size: Some(struct_size) };
                scope_data.structs.add_struct(&struct_definition);

                Some(ASTMetadata {
                    remaining_slice,
                    resultant_tree: struct_definition
                })
            },

            _ => Some(ASTMetadata { 
                remaining_slice: curr_queue_idx,
                resultant_tree: scope_data.structs.get_struct(&struct_name).unwrap().clone()//TODO this could declare a struct?
            })
        }
    }
}

fn try_consume_struct_member(tokens_queue: &TokenQueue, curr_queue_idx: &mut TokenQueueSlice, scope_data: &mut ParseData) -> Option<Declaration> {
    if curr_queue_idx.get_slice_size() == 0 {
        return None;
    }

    //consume the base type
    let ASTMetadata { remaining_slice, resultant_tree:base_type } = consume_base_type(tokens_queue, &curr_queue_idx, scope_data).unwrap();

    curr_queue_idx.index = remaining_slice.index;//consume it and let the calling function know

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
    struct_decls: HashMap<String, StructDefinition>//note: definition also contains a copy of the struct's name
}
impl StructList {
    pub fn new() -> StructList {
        StructList { struct_decls: HashMap::new() }
    }
    /**
     * gets my structs and appends the structs from other, overwriting any duplicates
     */
    pub fn merge(&self, other: &StructList) -> StructList {
        StructList { 
            //merge my and other structs, overwriting mine if there are duplicates
            struct_decls: self.struct_decls.clone().into_iter().chain(other.struct_decls.clone().into_iter()).collect()
        }
    }
    pub fn add_struct(&mut self, new_definition: &StructDefinition) {
        let new_struct_name = new_definition.get_name().as_ref().unwrap();

        if let Some(definition) = self.struct_decls.get_mut(new_struct_name) {
            match (&definition.ordered_members, &new_definition.ordered_members) {
                (Some(_), Some(_)) => panic!("redefinition of struct {}", definition.name.clone().unwrap()),

                (None, Some(_)) => definition.ordered_members = new_definition.ordered_members.clone(),//new definition contains more data

                _ => {}//new definition provides no new data
            }
        } else {
            self.struct_decls.insert(new_struct_name.to_string(), new_definition.clone());//add new struct
        }
    }

    /**
     * note: gets struct by name of struct, not by name of any variables
     */
    pub fn get_struct(&self, name: &str) -> Option<&StructDefinition> {
        self.struct_decls.get(name)
    }
}