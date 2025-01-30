use crate::{lexer::{token::Token, token_walk::TokenQueue}, statement::Statement, token::type_info::TypeInfo};


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
    pub fn try_consume_func_definition(tokens_queue: &mut TokenQueue) -> Option<FunctionDefinition> {

        tokens_queue.save_checkpoint();//if things go pear-shaped in this function, go back to this checkpoint to restore progress

        let mut return_data = Vec::new();

        //try and consume as many type specifiers as possible
        loop {
            if let Token::TYPESPECIFIER(ts) = tokens_queue.peek()? {
                return_data.push(ts.clone());
                tokens_queue.consume();
            } else {
                break;
            }
        }

        if return_data.len() == 0 {
            tokens_queue.pop_checkpoint();
            return None;//missing type info
        }

        //try to match an identifier, to find out the function name

        let func_name = 
        if let Token::IDENTIFIER(ident) = tokens_queue.consume()? {
            ident.to_string()
        }
        else {
            tokens_queue.pop_checkpoint();
            return None;
        };

        //pop the ( after the function name
        if Token::PUNCTUATION("(".to_owned()) != tokens_queue.consume()? {
            tokens_queue.pop_checkpoint();
            return None;
        }

        //skip over params for now (TODO function params)
        loop {
            if Token::PUNCTUATION(")".to_owned()) == tokens_queue.consume()? {
                break;
            }
        }

        //read the next statement (statement includes a scope)
        if let Some(function_code) = Statement::try_consume_statement(tokens_queue) {
            tokens_queue.pop_checkpoint();
            return Some(
                FunctionDefinition {
                    return_type:return_data,
                    function_name: func_name,
                    code: function_code
                }
            );
        }

        tokens_queue.pop_checkpoint();
        None
    }
}