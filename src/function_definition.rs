use crate::{asm_boilerplate, lexer::{token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, stack_parsing_info::StackInfo, statement::Statement, type_info::TypeInfo};
use std::fmt::Write;

/**
 * This is a definition of a function
 */
#[derive(Debug)]
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
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice) -> Option<(FunctionDefinition, TokenQueueSlice)> {
        let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        let mut return_data = Vec::new();
        let mut local_variables = StackInfo::new();//TODO

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

    pub fn generate_assembly(&self) -> String {
        //this uses a custom calling convention
        //all params passed on the stack, right to left (caller cleans these up)
        //return value in RAX
        let mut result = String::new();

        //set label as same as function name
        writeln!(result, "func_{}:", self.function_name).unwrap();
        //create stack frame
        writeln!(result, "push rbp").unwrap();
        writeln!(result, "mov rbp, rsp").unwrap();

        //TODO generate stack information, and pass it to the code
        write!(result, "{}", self.code.generate_assembly()).unwrap();

        writeln!(result, "{}", asm_boilerplate::func_exit_boilerplate()).unwrap();

        return result;
    }
}