use memory_size::MemoryLayout;

use crate::{ast_metadata::ASTMetadata, expression::Expression, lexer::{token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size, stack_variables::StackVariables, type_info::TypeInfo};
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct Declaration {
    data_type: Vec<TypeInfo>,
    name: String,
    initialisation: Option<Expression>
    //TODO: have a tree of declarator modifiers, because int *a[] is different from int (*a)[] and int *a() is a function pointer, etc
}

impl Declaration {
    pub fn get_type(&self) -> &Vec<TypeInfo> {
        &self.data_type
    }
    pub fn get_memory_usage(&self) -> MemoryLayout {
        return MemoryLayout::from_bytes(8);//default for all data is 64 bits
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
    
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, _local_variables: &StackVariables) -> Option<ASTMetadata<Vec<Declaration>>> {
        let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        let mut data_type_info = Vec::new();
        let mut declarations = Vec::new();
        let mut extra_stack_needed = MemoryLayout::new();
        
        //try and consume as many type specifiers as possible
        loop {
            if let Token::TYPESPECIFIER(ts) = tokens_queue.peek(&curr_queue_idx)? {
                data_type_info.push(ts.clone());
                tokens_queue.consume(&mut curr_queue_idx);
            } else {
                break;
            }
        }

        if data_type_info.len() == 0 {
            return None;//missing type info
        }

        //find semicolon
        let semicolon_idx = tokens_queue.find_closure_in_slice(&curr_queue_idx, false, |x| *x == Token::PUNCTUATION(";".to_owned()))?;
        //find where all the declarators are (the x=2,y part in int x=2,y;)
        let all_declarators_segment = TokenQueueSlice{index:curr_queue_idx.index, max_index:semicolon_idx.index};
        //split each declarator
        let declarator_segments = tokens_queue.split_outside_parentheses(&all_declarators_segment, |x| *x == Token::PUNCTUATION(",".to_owned()));

        for declarator_segment in declarator_segments {
            //try and consume the declarator
            if let Some(ASTMetadata { remaining_slice: _, resultant_tree, extra_stack_used }) = try_consume_declarator(tokens_queue, &declarator_segment, _local_variables, &data_type_info) {
                declarations.push(resultant_tree);//the declarator consumption actaully gives us a full declaration
                extra_stack_needed += extra_stack_used;
            }
        }

        curr_queue_idx = TokenQueueSlice{index: semicolon_idx.index + 1, max_index: curr_queue_idx.max_index};//consume the semicolon

        assert!(declarations.len() > 0);//must have at least one declaration

        Some(ASTMetadata {
            resultant_tree: declarations,
            remaining_slice: curr_queue_idx,
            extra_stack_used: extra_stack_needed
        })
    }
    
    pub fn generate_assembly(&self) -> String {
        let mut result = String::new();

        if let Some(init) = &self.initialisation {
            write!(result, "{}", init.generate_assembly()).unwrap();//init is an expression that assigns to the variable, so no more work for me
        }

        result
    }

    
}

/**
 * claims to consume a declarator, but actaully takes in the data type too, and gives back a full declaration
 */
pub fn try_consume_declarator(tokens_queue: &mut TokenQueue, slice: &TokenQueueSlice, local_variables: &StackVariables, data_type: &Vec<TypeInfo>) -> Option<ASTMetadata<Declaration>> {
    let mut curr_queue_idx = slice.clone();

    let var_name = 
    if let Token::IDENTIFIER(ident) = tokens_queue.consume(&mut curr_queue_idx)? {
        ident.to_string()
    }
    else {
        return None;
    };

    let extra_stack_needed = MemoryLayout::from_bytes(8);//default for all data is 64 bits

    //try to match an initialisation expression
    let initialisation = 
    match tokens_queue.peek(&curr_queue_idx) {
        Some(Token::PUNCTUATION(p)) if p == "=".to_string() => {
            tokens_queue.consume(&mut curr_queue_idx).unwrap();//consume the equals sign

            //consume the right hand side of the initialisation
            //then create an assignment expression to write the value to the variable
            Some(Expression::BINARYEXPR(
                Box::new(Expression::STACKVAR(local_variables.get_variable_bp_offset(&var_name).unwrap())),//TODO this variable hasn't been put in local_variables yet, so this will fail
                crate::operator::Operator::ASSIGN,
                Box::new(Expression::try_consume_whole_expr(tokens_queue, &curr_queue_idx, local_variables)?)
            ))
            
        },
        _ => None
    };

    Some(ASTMetadata {
        resultant_tree: Declaration {
            name: var_name,
            initialisation,
            data_type: data_type.clone(),
        }, 
        remaining_slice: TokenQueueSlice::empty(),
        extra_stack_used: extra_stack_needed
    })
}