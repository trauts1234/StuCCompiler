use crate::{asm_gen_data::AsmData, assembly::{assembly::Assembly, operand::{immediate::MemorySizeExt, register::Register, Operand, RegOrMem}, operation::AsmOperation}, ast_metadata::ASTMetadata, data_type::{base_type::BaseType, recursive_data_type::DataType}, debugging::DebugDisplay, declaration::Declaration, expression::Expression, expression_visitors::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor, reference_assembly_visitor::ReferenceVisitor}, initialised_declaration::{consume_base_type, try_consume_declaration_modifiers}, lexer::{keywords::Keyword, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, parse_data::ParseData};
use unwrap_let::unwrap_let;
use memory_size::MemorySize;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum StructIdentifier {
    NAME(String),//struct has been given a name
    ID(i32)//anonymous struct has been given an id number
}
impl DebugDisplay for StructIdentifier {
    fn display(&self) -> String {
        match self {
            StructIdentifier::NAME(x) => x.to_owned(),
            StructIdentifier::ID(x) => format!("ID({})", x),
        }
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
    pub fn pad_members(&self, asm_data: &AsmData) -> StructDefinition {
        let mut current_offset = MemorySize::new();

        let mut result = Vec::new();

        for m in self.ordered_members.as_ref().expect("tried to create struct with no members") {
            let alignment_bytes = calculate_alignment(m.get_type(), asm_data).size_bytes();

            let bytes_past_last_boundary = current_offset.size_bytes() % alignment_bytes;
            let extra_padding = (alignment_bytes - bytes_past_last_boundary) % alignment_bytes;
            current_offset += MemorySize::from_bytes(extra_padding);//increase offset in this struct to reach optimal alignment

            result.push((m.clone(), current_offset));
            current_offset += m.get_type().memory_size(asm_data);//increase offset in struct by the size of the member
        }

        //lastly, align to largest member's alignment, so that if this struct is in an array, subsequent structs are aligned
        let largest_member_alignment = self.ordered_members.as_ref().unwrap().iter()
            .map(|x| calculate_alignment(x.get_type(), asm_data))
            .fold(MemorySize::new(), |acc, x| acc.max(x))
            .size_bytes();
        let bytes_past_last_boundary = current_offset.size_bytes() % largest_member_alignment;
        let extra_padding = (largest_member_alignment - bytes_past_last_boundary) % largest_member_alignment;
        current_offset += MemorySize::from_bytes(extra_padding);

        StructDefinition { ordered_members: Some(result), size: Some(current_offset) }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StructDefinition {
    ordered_members: Option<Vec<(Declaration, MemorySize)>>,//decl and offset from start that this member is located
    size: Option<MemorySize>
}

#[derive(Clone, Debug)]
pub struct StructMemberAccess {
    struct_tree: Box<Expression>,//need a tree or something to represent what gives me back the struct
    member_name: String,
}

impl StructMemberAccess {
    pub fn new(struct_tree: Expression, member_name: String) -> StructMemberAccess {
        StructMemberAccess { struct_tree: Box::new(struct_tree), member_name }
    }

    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_struct_member_access(self)
    }

    pub fn get_base_struct_tree(&self) -> &Expression {
        &self.struct_tree
    }
    pub fn get_member_name(&self) -> &str {
        &self.member_name
    }

    pub fn get_data_type(&self, asm_data: &AsmData) -> DataType {
        let struct_tree_type = self.struct_tree.accept(&mut GetDataTypeVisitor {asm_data});//get type of the tree that returns the struct

        unwrap_let!(DataType::RAW(BaseType::STRUCT(struct_name)) = struct_tree_type);

        let (member_decl, _) = asm_data.get_struct(&struct_name).get_member_data(&self.member_name);//get the type of the member

        member_decl.get_type().clone()
    }

    pub fn put_addr_in_acc(&self, asm_data: &AsmData, stack_data: &mut MemorySize) -> Assembly {
        let mut result = Assembly::make_empty();

        result.add_comment(format!("getting address of struct's member {}", self.member_name));
        //put tree's address in acc
        //add the member offset

        let base_struct_address_asm = self.struct_tree.accept(&mut ReferenceVisitor {asm_data, stack_data});//assembly to get address of struct

        let base_struct_type = self.struct_tree.accept(&mut GetDataTypeVisitor {asm_data});//get type of the tree that returns the struct

        unwrap_let!(DataType::RAW(BaseType::STRUCT(struct_name)) = base_struct_type);

        let (_, struct_member_offset) = asm_data.get_struct(&struct_name).get_member_data(&self.member_name);//get offset for the specific member

        //get address of struct
        result.merge(&base_struct_address_asm);

        //go up by member offset
        result.add_instruction(AsmOperation::ADD {
            destination: RegOrMem::Reg(Register::acc()),
            increment: Operand::Imm(struct_member_offset.as_imm()),
            data_type: DataType::RAW(BaseType::U64)//pointer addition is u64 add
        });

        result
    }
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
    
    pub fn try_consume_struct_as_type(tokens_queue: &TokenQueue, previous_slice: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<ASTMetadata<(StructIdentifier, UnpaddedStructDefinition)>> {

        let mut curr_queue_idx = previous_slice.clone();

        if tokens_queue.consume(&mut curr_queue_idx, &scope_data)? != Token::KEYWORD(Keyword::STRUCT) {
            return None;//needs preceding "struct"
        }
    
        let struct_name = if let Token::IDENTIFIER(x) = tokens_queue.peek(&mut curr_queue_idx, &scope_data).unwrap() {
            tokens_queue.consume(&mut curr_queue_idx, scope_data).unwrap();//consume the name
            StructIdentifier::NAME(x)
        } else {
            StructIdentifier::ID(scope_data.generate_struct_id())
        };

        match tokens_queue.peek(&curr_queue_idx, &scope_data).unwrap() {
            Token::PUNCTUATOR(Punctuator::OPENSQUIGGLY) => {
                let close_squiggly_idx = tokens_queue.find_matching_close_bracket(curr_queue_idx.index);
                let mut inside_variants = TokenQueueSlice{index:curr_queue_idx.index+1, max_index: close_squiggly_idx};//+1 to skip the {
                let remaining_slice = TokenQueueSlice{index:close_squiggly_idx+1, max_index:curr_queue_idx.max_index};

                let mut members = Vec::new();
                while inside_variants.get_slice_size() > 0 {
                    let mut new_member = try_consume_struct_member(tokens_queue, &mut inside_variants, scope_data);
                    members.append(&mut new_member);
                }

                assert!(inside_variants.get_slice_size() == 0);//must consume all tokens in variants

                let struct_definition = UnpaddedStructDefinition { ordered_members: Some(members),  };
                scope_data.add_struct(&struct_name, &struct_definition);

                Some(ASTMetadata {
                    remaining_slice,
                    resultant_tree: (struct_name, struct_definition)
                })
            },

            _ => Some(ASTMetadata { 
                remaining_slice: curr_queue_idx,
                resultant_tree: (struct_name.clone(), scope_data.get_struct(&struct_name).unwrap().clone())//TODO this could declare a struct?
            })
        }
    }
}

///in struct definitions, this will consume the `int a,b;` part of `struct {int a,b;char c;}`
fn try_consume_struct_member(tokens_queue: &TokenQueue, curr_queue_idx: &mut TokenQueueSlice, scope_data: &mut ParseData) -> Vec<Declaration> {

    //consume the base type
    let ASTMetadata { remaining_slice, resultant_tree:base_type } = consume_base_type(tokens_queue, &curr_queue_idx, scope_data).unwrap();

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

fn calculate_alignment(data_type: &DataType, asm_data: &AsmData) -> MemorySize {
    if let DataType::ARRAY {..} = data_type {
        calculate_alignment(&data_type.remove_outer_modifier(), asm_data) //array of x should align to a boundary of sizeof x, but call myself recursively to handle 2d arrays
    } else {
        data_type.memory_size(asm_data)
    }
}