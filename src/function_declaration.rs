use crate::{ast_metadata::ASTMetadata, data_type::{base_type::BaseType, recursive_data_type::RecursiveDataType, type_modifier::DeclModifier}, declaration::{consume_base_type, try_consume_declaration_modifiers, Declaration}, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, parse_data::ParseData};

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub(crate) function_name: String,
    pub(crate) params: Vec<Declaration>,//should this be a data type?
    pub(crate) return_type: RecursiveDataType,
}

impl FunctionDeclaration {
    /**
     * detects whether the function has extern linkage
     */
    pub fn external_linkage(&self) -> bool {
        true//extern or not, this has external linkage
    }

    /**
     * consumes a function declaration only, and will return None if the function has a definition attached
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<ASTMetadata<FunctionDeclaration>> {
        let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        let ASTMetadata { remaining_slice, resultant_tree: decl, .. } = consume_decl_only(tokens_queue, &curr_queue_idx, scope_data)?;

        curr_queue_idx = remaining_slice;//skip decl as it has now been parsed

        if Token::PUNCTUATOR(Punctuator::SEMICOLON) != tokens_queue.consume(&mut curr_queue_idx, &scope_data)? {
            //TODO what if an enum was generated, then this check fails???
            //the enum would be created but shouldn't have been
            return None;//no trailing semicolon
        }

        Some(ASTMetadata {
            resultant_tree: decl,
            remaining_slice: curr_queue_idx,
        })
    } 
}

/**
 * consumes a function declaration, but NOT the trailing semicolon or function definition
 * parses the int f(int x) part of:
 *  int f(int x);
 *  int f(int x) {return 1;}
 */
pub fn consume_decl_only(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<ASTMetadata<FunctionDeclaration>> {

    let mut return_modifiers = Vec::new();

    let ASTMetadata { remaining_slice, resultant_tree:return_data_type } = consume_base_type(tokens_queue, previous_queue_idx, scope_data)?;

    let mut curr_queue_idx = remaining_slice.clone();

    while Token::PUNCTUATOR(Punctuator::ASTERISK) == tokens_queue.peek(&curr_queue_idx, &scope_data)? {
        return_modifiers.push(DeclModifier::POINTER);
        tokens_queue.consume(&mut curr_queue_idx, &scope_data);
    }

    //try to match an identifier, to find out the function name

    let func_name = 
    if let Token::IDENTIFIER(ident) = tokens_queue.consume(&mut curr_queue_idx, &scope_data)? {
        ident.to_string()
    }
    else {
        return None;
    };

    //check that there is a bracket round the params
    if Token::PUNCTUATOR(Punctuator::OPENCURLY) != tokens_queue.peek(&mut curr_queue_idx, &scope_data)? {
        return None;
    }

    //find the brackets enclosing the params
    let args_location = TokenQueueSlice { 
        index: curr_queue_idx.index + 1,//+1 to avoid storing the open bracket in the sub-expression 
        max_index: tokens_queue.find_matching_close_bracket(curr_queue_idx.index) 
    };

    tokens_queue.consume(&mut curr_queue_idx, &scope_data).unwrap();//consume the open bracket

    let args_segments = tokens_queue.split_outside_parentheses(&args_location, |x| *x == Token::PUNCTUATOR(Punctuator::COMMA));

    //grab all the args
    let mut args = Vec::new();
    if args_location.get_slice_size() >= 1{//ensure there is text between the brackets
        for arg_segment in args_segments {
            args.push(consume_fn_param(tokens_queue, &arg_segment, scope_data)?);
        }
    }

    curr_queue_idx.index = args_location.max_index;//jump to end of args

    //pop the ) at the end of the params
    if Token::PUNCTUATOR(Punctuator::CLOSECURLY) != tokens_queue.consume(&mut curr_queue_idx, &scope_data)? {
        return None;
    }

    //this does not consume anything else, so could consume the start of a declaration OR a definition

    return Some(ASTMetadata{
        resultant_tree: 
        FunctionDeclaration {
            function_name: func_name,
            params: args,
            return_type: RecursiveDataType::new_from_slice(return_data_type, &return_modifiers),
        },
        remaining_slice: curr_queue_idx});
}

fn consume_fn_param(tokens_queue: &mut TokenQueue, arg_segment: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<Declaration> {
    let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(arg_segment);

    if Token::PUNCTUATOR(Punctuator::ELIPSIS) == tokens_queue.peek(&curr_queue_idx, &scope_data)? {
        tokens_queue.consume(&mut curr_queue_idx, &scope_data);
        return Some(Declaration { data_type: 
            RecursiveDataType::new(BaseType::VaArg),
             name: String::new()//va arg has no name 
        })
    }

    let ASTMetadata { remaining_slice, resultant_tree:data_type_base } = consume_base_type(tokens_queue, &mut curr_queue_idx, scope_data).unwrap();
    let curr_queue_idx = remaining_slice.clone();

    //by parsing the *x[2] part of int *x[2];, I can get the modifiers and the variable name
    let ASTMetadata{
        resultant_tree: Declaration { data_type: full_data_type, name: var_name },
        remaining_slice:_,
    } = try_consume_declaration_modifiers(tokens_queue, &curr_queue_idx, &data_type_base, scope_data)?;

    Some(Declaration {
        data_type: full_data_type.decay(),//.decay since arrays ALWAYS decay to pointers, even when sizeof is involved
        name: var_name
    })
}