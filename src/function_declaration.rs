use crate::{ast_metadata::ASTMetadata, compilation_state::label_generator::LabelGenerator, data_type::{base_type::BaseType, recursive_data_type::DataType, storage_type::StorageDuration, type_modifier::DeclModifier}, debugging::DebugDisplay, declaration::Declaration, initialised_declaration::{consume_type_specifier, try_consume_declaration_modifiers}, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, parse_data::ParseData};

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
    pub(crate) function_name: String,
    pub(crate) params: Vec<Declaration>,//should this be a data type?
    pub(crate) return_type: DataType,
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
    pub fn try_consume(tokens_queue: &TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData, struct_label_gen: &mut LabelGenerator) -> Option<ASTMetadata<FunctionDeclaration>> {
        let mut curr_queue_idx = previous_queue_idx.clone();

        let ASTMetadata { remaining_slice, resultant_tree: decl, .. } = consume_decl_only(tokens_queue, &curr_queue_idx, scope_data, struct_label_gen)?;

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
pub fn consume_decl_only(tokens_queue: &TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData, struct_label_gen: &mut LabelGenerator) -> Option<ASTMetadata<FunctionDeclaration>> {

    //try and consume int* or similar as return type, get curr_queue_idx and the function return type
    // the return value's storage duration (static, extern etc.) is the visibility of the function?
    let ASTMetadata { remaining_slice: mut curr_queue_idx, resultant_tree: (return_type, func_visibility) } = consume_fully_qualified_type(tokens_queue, previous_queue_idx, scope_data, struct_label_gen)?;

    //try to match an identifier, to find out the function name

    let function_name = 
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

    let args_segments = tokens_queue.split_outside_parentheses(&args_location, |x| *x == Token::PUNCTUATOR(Punctuator::COMMA), &TokenSearchType::skip_all());

    //grab all the args
    let mut params = Vec::new();
    if args_location.get_slice_size() >= 1{//ensure there is text between the brackets
        for arg_segment in args_segments {
            params.push(consume_fn_param(tokens_queue, &arg_segment, scope_data, struct_label_gen)?);
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
            function_name,
            params,
            return_type
        },
        remaining_slice: curr_queue_idx});
}

fn consume_fn_param(tokens_queue: &TokenQueue, arg_segment: &TokenQueueSlice, scope_data: &mut ParseData, struct_label_gen: &mut LabelGenerator) -> Option<Declaration> {
    let mut curr_queue_idx = arg_segment.clone();

    if Token::PUNCTUATOR(Punctuator::ELIPSIS) == tokens_queue.peek(&curr_queue_idx, &scope_data)? {
        tokens_queue.consume(&mut curr_queue_idx, &scope_data);
        return Some(Declaration { data_type: 
            DataType::new(BaseType::VaArg),
             name: String::new()//va arg has no name 
        })
    }

    println!("{:?}", tokens_queue.get_slice(&curr_queue_idx));

    let ASTMetadata { remaining_slice, resultant_tree: (data_type_base, storage_class) } = consume_type_specifier(tokens_queue, &mut curr_queue_idx, scope_data, struct_label_gen).unwrap();
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

//TODO move to more appropriate file
pub fn consume_fully_qualified_type(tokens_queue: &TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData, struct_label_gen: &mut LabelGenerator) -> Option<ASTMetadata<(DataType, StorageDuration)>> {
    let mut return_modifiers = Vec::new();

    let ASTMetadata { remaining_slice, resultant_tree: (return_data_type, storage_duration) } = consume_type_specifier(tokens_queue, previous_queue_idx, scope_data, struct_label_gen)?;

    let mut curr_queue_idx = remaining_slice.clone();

    while Some(Token::PUNCTUATOR(Punctuator::ASTERISK)) == tokens_queue.peek(&curr_queue_idx, &scope_data) {
        return_modifiers.push(DeclModifier::POINTER);
        tokens_queue.consume(&mut curr_queue_idx, &scope_data);
    }

    Some(ASTMetadata {
        remaining_slice: curr_queue_idx,
        resultant_tree: (DataType::new_from_slice(return_data_type, &return_modifiers), storage_duration),
    })
}

impl DebugDisplay for FunctionDeclaration {
    fn display(&self) -> String {
        format!(
            "function {}: {} -> {}",
            self.function_name,
            self.params.iter().map(|x| x.display()).collect:: <Vec<_>>().join(", "),
            self.return_type.display()
        )
    }
}