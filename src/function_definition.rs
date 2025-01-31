use crate::{lexer::{token::Token, token_savepoint::TokenQueueLocation, token_walk::TokenQueue}, statement::Statement, token::type_info::TypeInfo};


/**
 * This is a definition of a function
 */
pub struct FunctionDefinition {
    return_type: Vec<TypeInfo>,
    function_name: String,
    code: Statement//statement could be a scope if it wants
    //params: Declaration,
    
}

impl FunctionDefinition {
    /**
     * consumes tokens to try and make a function definition
     * returns some(function found, remaining tokens) if found, else None
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueLocation) -> Option<(FunctionDefinition, TokenQueueLocation)> {
        let mut curr_queue_idx = TokenQueueLocation::from_previous_savestate(previous_queue_idx);

        let mut return_data = Vec::new();

        //try and consume as many type specifiers as possible
        loop {
            if let Token::TYPESPECIFIER(ts) = tokens_queue.peek(&curr_queue_idx)? {
                return_data.push(ts.clone());
                tokens_queue.consume(&mut curr_queue_idx);
            } else {
                break;
            }
        }

        if return_data.len() == 0 {
            return None;//missing type info
        }

        //try to match an identifier, to find out the function name

        let func_name = 
        if let Token::IDENTIFIER(ident) = tokens_queue.consume(&mut curr_queue_idx)? {
            ident.to_string()
        }
        else {
            return None;
        };

        //pop the ( after the function name
        if Token::PUNCTUATION("(".to_owned()) != tokens_queue.consume(&mut curr_queue_idx)? {
            return None;
        }

        //skip over params for now (TODO function params)
        loop {
            if Token::PUNCTUATION(")".to_owned()) == tokens_queue.consume(&mut curr_queue_idx)? {
                break;
            }
        }

        //read the next statement (statement includes a scope)
        let (function_code, remaining_tokens_idx) = Statement::try_consume(tokens_queue, &curr_queue_idx)?;
        
        return Some((
            FunctionDefinition {
                return_type:return_data,
                function_name: func_name,
                code: function_code
            },
            remaining_tokens_idx));
    }
}