use crate::{lexer::token::Token, statement::Statement, token::type_info::TypeInfo};


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
    pub fn try_consume_func_definition(tokens: &Vec<Token>) -> Option<(FunctionDefinition, Vec<Token>)> {
        let mut tokens_clone = tokens.clone();//so we can mess with it

        //TODO try to pop type infos
        //then pop an identifier, check it is not a keyword
        //then pop a "("
        //then pop until ")" as params are not implemented
        //then call to pop a Statement (should usually match a scope)
        //then return Some(definition, tokens_clone)
        todo!()
    }
}