use unwrap_let::unwrap_let;

use crate::{asm_gen_data::AsmData, ast_metadata::ASTMetadata, constexpr_parsing::ConstexprValue, data_type::{base_type::BaseType, recursive_data_type::RecursiveDataType}, declaration::{consume_base_type, try_consume_declaration_modifiers, Declaration}, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, number_literal::NumberLiteral, parse_data::ParseData};


pub struct GlobalVariable {
    decl: Declaration,
    default_value: ConstexprValue//perhaps some more abstract data type when structs are implemented
}

impl GlobalVariable {
    pub fn generate_assembly(&self, asm_data: &AsmData) -> String {
        match &self.default_value {
            ConstexprValue::NUMBER(number_literal) => {
                unwrap_let!(RecursiveDataType::RAW(decl_underlying_type) = &self.decl.data_type);

                format!("{} db {}\n", 
                    self.decl.get_name(), 
                    number_literal.cast(decl_underlying_type).get_comma_separated_bytes(asm_data)//cast the number to the variable's type, then write the bytes for it
                )
            },
            ConstexprValue::STRING(string_literal) => format!("{} db {}\n", self.decl.get_name(), string_literal.get_comma_separated_bytes()),
            ConstexprValue::POINTER(to_dereference) => format!("{} dq {}\n", self.decl.get_name(), to_dereference),
        }
    }

    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<ASTMetadata<Vec<GlobalVariable>>> {

        let mut declarations = Vec::new();
        
        //consume int or unsigned int or enum etc.
        let ASTMetadata { remaining_slice, resultant_tree:base_type } = consume_base_type(tokens_queue, previous_queue_idx, scope_data)?;

        let mut curr_queue_idx = remaining_slice.clone();

        //find semicolon
        let semicolon_idx = tokens_queue.find_closure_matches(&curr_queue_idx, false, |x| *x == Token::PUNCTUATOR(Punctuator::SEMICOLON), &TokenSearchType { skip_in_curly_brackets: false, skip_in_square_brackets: false, skip_in_squiggly_brackets:true })?;
        //find where all the declarators are (the x=2,y part in int x=2,y;)
        let all_declarators_segment = TokenQueueSlice{index:curr_queue_idx.index, max_index:semicolon_idx.index};
        //split each declarator
        let declarator_segments = tokens_queue.split_outside_parentheses(&all_declarators_segment, |x| *x == Token::PUNCTUATOR(Punctuator::COMMA));

        for declarator_segment in declarator_segments {
            //try and consume the declarator
            if let Some(ASTMetadata { resultant_tree, .. }) = try_consume_constexpr_declarator(tokens_queue, &declarator_segment, &base_type, scope_data) {
                declarations.push(resultant_tree);//the declarator consumption actaully gives us a full declaration
            }
        }

        curr_queue_idx = TokenQueueSlice{index: semicolon_idx.index + 1, max_index: curr_queue_idx.max_index};//consume the semicolon

        Some(ASTMetadata {
            resultant_tree: declarations,
            remaining_slice: curr_queue_idx,
        })
    }
}

fn try_consume_constexpr_declarator(tokens_queue: &mut TokenQueue, slice: &TokenQueueSlice, base_type: &BaseType, scope_data: &mut ParseData) -> Option<ASTMetadata<GlobalVariable>> {
    if slice.get_slice_size() == 0 {
        return None;
    }
    
    let mut curr_queue_idx = slice.clone();
    
    let ASTMetadata{resultant_tree: Declaration { data_type, name: var_name }, remaining_slice:remaining_tokens} = try_consume_declaration_modifiers(tokens_queue, &curr_queue_idx, base_type, scope_data)?;

    scope_data.add_variable(&var_name, data_type.clone());//save variable to variable list early, so that I can reference it in the initialisation

    let decl = Declaration {
        name: var_name.to_string(),
        data_type
    };

    curr_queue_idx = remaining_tokens;//tokens have been consumed

    //try to match an initialisation expression
    let default_value = consume_constexpr_initialisation(tokens_queue, &mut curr_queue_idx, scope_data);

    Some(ASTMetadata {
        resultant_tree: GlobalVariable {
            decl,
            default_value,
        }, 
        remaining_slice: TokenQueueSlice::empty(),
    })
}

fn consume_constexpr_initialisation(tokens_queue: &mut TokenQueue, curr_queue_idx: &mut TokenQueueSlice, scope_data: &mut ParseData) -> ConstexprValue {
    if tokens_queue.peek(&curr_queue_idx, &scope_data) != Some(Token::PUNCTUATOR(Punctuator::EQUALS)){
        return ConstexprValue::NUMBER(NumberLiteral::new("0"));
    }

    tokens_queue.consume(curr_queue_idx, &scope_data).unwrap();//consume the equals sign

    ConstexprValue::try_consume_whole_constexpr(tokens_queue, curr_queue_idx, scope_data).unwrap()//return the consumed value for the variable
}