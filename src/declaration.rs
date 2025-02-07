use memory_size::MemoryLayout;

use crate::{ast_metadata::ASTMetadata, expression::Expression, lexer::{token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size, stack_variables::StackVariables, type_info::TypeInfo};
use std::fmt::Write;

#[derive(Debug)]
pub struct Declaration {
    data_type: Vec<TypeInfo>,
    var_name: String,
    bytes_sub_from_bp: MemoryLayout,//the number of qwords subtracted from RBP
    initialisation: Option<Expression>//for int x=0; the declaration is split into int x; x=0;
}

impl Declaration {
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

        //find a slice that has the initialisation expression (the x=0 bit from int x=0;)
        //warning: int *x=0; is not the same as int *x; *x=0; as you are setting the pointer address to 0 in the first example but setting the pointer value in the second
        let mut expression_slice = curr_queue_idx.clone();
        let semicolon_idx = tokens_queue.find_closure_in_slice(&expression_slice, false, |x| *x == Token::PUNCTUATION(";".to_owned()))?;
        expression_slice.copy_end_index(&semicolon_idx);

        //try to match an identifier, to find out the variable

        let var_name = 
        if let Token::IDENTIFIER(ident) = tokens_queue.consume(&mut curr_queue_idx)? {
            ident.to_string()
        }
        else {
            return None;
        };

        //todo handle comma separated declarations
        Some(ASTMetadata {
            resultant_tree: Declaration {
                data_type: data_type_info,
                var_name,
                bytes_sub_from_bp: todo!(),//TODO propogate variables upwards
                initialisation: Expression::try_consume_whole_expr(tokens_queue, &expression_slice, local_variables)//pointers break this maybe
            }, 
            remaining_slice: curr_queue_idx
        })
    }
    
    pub fn generate_assembly(&self) -> String {
        let mut result = String::new();

        //assign value to the variable, if one is required
        match &self.initialisation{
            Some(init) => {write!(result, "{}", init.generate_assembly()).unwrap()},
            None => {}
        };

        result
    }

    
}