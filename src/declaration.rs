use memory_size::MemoryLayout;

use crate::{ast_metadata::ASTMetadata, lexer::{token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size, stack_variables::StackVariables, type_info::TypeInfo};
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct Declaration {
    data_type: Vec<TypeInfo>,
    var_name: String,
    bytes_sub_from_bp: MemoryLayout,//the number of qwords subtracted from RBP
    //initialisation: Option<Expression>//for int x=0; the declaration is split into int x; x=0;
}

impl Declaration {
    pub fn get_name(&self) -> &str {
        &self.var_name
    }
    pub fn get_type(&self) -> &Vec<TypeInfo> {
        &self.data_type
    }
    pub fn get_memory_usage(&self) -> MemoryLayout {
        MemoryLayout::from_bytes(8)
    }
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, local_variables: &StackVariables) -> Option<ASTMetadata<Declaration>> {
        let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        let mut data_type_info = Vec::new();
        
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
        
        //try to match an identifier, to find out the variable

        let var_name = 
        if let Token::IDENTIFIER(ident) = tokens_queue.consume(&mut curr_queue_idx)? {
            ident.to_string()
        }
        else {
            return None;
        };

        tokens_queue.consume(&mut curr_queue_idx);//consume the semicolon

        let extra_stack_needed = MemoryLayout::from_bytes(8);//same as get_memory_usage, default for now
        let var_sub_from_bp = local_variables.get_stack_used() + extra_stack_needed;//how far from bp is this variable

        //todo handle comma separated declarations
        Some(ASTMetadata {
            resultant_tree: Declaration {
                data_type: data_type_info,
                var_name,
                bytes_sub_from_bp: var_sub_from_bp,
                //initialisation: Expression::try_consume_whole_expr(tokens_queue, &expression_slice, local_variables)//pointers break this maybe
            }, 
            remaining_slice: curr_queue_idx,
            extra_stack_used: extra_stack_needed
        })
    }
    
    pub fn generate_assembly(&self) -> String {
        String::new()

        //assign value to the variable, if one is required
        /*match &self.initialisation{
            Some(init) => {write!(result, "{}", init.generate_assembly()).unwrap()},
            None => {}
        };*/
    }

    
}