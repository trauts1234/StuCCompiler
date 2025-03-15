use crate::{ast_metadata::ASTMetadata, constexpr_parsing::consume_whole_constexpr, data_type::{base_type::BaseType, data_type::DataType}, declaration::{consume_base_type, try_consume_declaration_modifiers, Declaration}, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size::MemoryLayout, number_literal::NumberLiteral, scope_data::ScopeData};


pub struct GlobalVariable {
    decl: Declaration,
    default_value: NumberLiteral//perhaps some more abstract data type when structs are implemented
}

impl GlobalVariable {
    pub fn get_name(&self) -> &Declaration {
        &self.decl
    }
    pub fn get_default_value(&self) -> &NumberLiteral {
        &self.default_value
    }
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ScopeData) -> Option<ASTMetadata<Vec<GlobalVariable>>> {
        let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        let mut declarations = Vec::new();
        
        //consume int or unsigned int or enum etc.
        let base_type = consume_base_type(tokens_queue, &mut curr_queue_idx, scope_data)?;

        //find semicolon
        let semicolon_idx = tokens_queue.find_closure_in_slice(&curr_queue_idx, false, |x| *x == Token::PUNCTUATOR(Punctuator::SEMICOLON))?;
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
            extra_stack_used: MemoryLayout::new()//global variables do not use the stack
        })
    }
}

fn try_consume_constexpr_declarator(tokens_queue: &mut TokenQueue, slice: &TokenQueueSlice, base_type: &BaseType, scope_data: &mut ScopeData) -> Option<ASTMetadata<GlobalVariable>> {
    let mut curr_queue_idx = slice.clone();
    
    let ASTMetadata{resultant_tree: Declaration { data_type: modifiers, name: var_name }, remaining_slice:remaining_tokens, extra_stack_used:_} = try_consume_declaration_modifiers(tokens_queue, &curr_queue_idx, base_type, scope_data)?;
    
    let data_type = DataType::new_from_base_type(&base_type, modifiers.get_modifiers());

    let extra_stack_needed = data_type.memory_size();//get the size of this variable

    let decl = Declaration {
        name: var_name.to_string(),
        data_type
    };

    scope_data.stack_vars.add_global_variable(decl.clone());//save variable to variable list early, so that I can reference it in the initialisation

    curr_queue_idx = remaining_tokens;//tokens have been consumed

    //try to match an initialisation expression
    let default_value = consume_constexpr_initialisation(tokens_queue, &mut curr_queue_idx, scope_data);

    Some(ASTMetadata {
        resultant_tree: GlobalVariable {
            decl,
            default_value,
        }, 
        remaining_slice: TokenQueueSlice::empty(),
        extra_stack_used: extra_stack_needed
    })
}

fn consume_constexpr_initialisation(tokens_queue: &mut TokenQueue, curr_queue_idx: &mut TokenQueueSlice, scope_data: &mut ScopeData) -> NumberLiteral {
    if tokens_queue.peek(&curr_queue_idx, &scope_data) != Some(Token::PUNCTUATOR(Punctuator::EQUALS)){
        return NumberLiteral::new("0");
    }

    tokens_queue.consume(curr_queue_idx, &scope_data).unwrap();//consume the equals sign

    consume_whole_constexpr(tokens_queue, curr_queue_idx, scope_data).unwrap()//return the consumed value for the variable
}