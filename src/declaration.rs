use memory_size::MemoryLayout;

use crate::{asm_generation::asm_line, ast_metadata::ASTMetadata, expression::Expression, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size, stack_variables::StackVariables, type_info::{DataType, DeclModifier, TypeInfo}};
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct InitialisedDeclaration{
    decl: Declaration,
    initialisation: Option<Expression>,
}

#[derive(Debug, Clone)]
pub struct Declaration {
    pub(crate) data_type: DataType,
    pub(crate) name: String,
}

#[derive(Debug, Clone)]
pub struct AddressedDeclaration {
    pub(crate) decl: Declaration,
    pub(crate) stack_offset: MemoryLayout
}

impl InitialisedDeclaration {
    /**
     * local_variables is mut as variables are added
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, _local_variables: &mut StackVariables) -> Option<ASTMetadata<Vec<InitialisedDeclaration>>> {
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
        let semicolon_idx = tokens_queue.find_closure_in_slice(&curr_queue_idx, false, |x| *x == Token::PUNCTUATOR(Punctuator::SEMICOLON))?;
        //find where all the declarators are (the x=2,y part in int x=2,y;)
        let all_declarators_segment = TokenQueueSlice{index:curr_queue_idx.index, max_index:semicolon_idx.index};
        //split each declarator
        let declarator_segments = tokens_queue.split_outside_parentheses(&all_declarators_segment, |x| *x == Token::PUNCTUATOR(Punctuator::COMMA));

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
            asm_line!(result, "{}", init.generate_assembly());//init is an expression that assigns to the variable, so no more work for me
        }

        result
    }
}

impl Declaration {
    pub fn get_type(&self) -> &DataType {
        //maybe unused
        &self.data_type
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }

    
}

/**
 * claims to consume a declarator, but actaully takes in the data type too, and gives back a full declaration
 */
pub fn try_consume_declarator(tokens_queue: &mut TokenQueue, slice: &TokenQueueSlice, local_variables: &mut StackVariables, data_type: &Vec<TypeInfo>) -> Option<ASTMetadata<InitialisedDeclaration>> {
    let mut curr_queue_idx = slice.clone();

    //by parsing the *x[2] part of int *x[2];, I can get the modifiers and the variable name
    let ASTMetadata{resultant_tree: Declaration { data_type: modifiers, name: var_name }, remaining_slice:remaining_tokens, extra_stack_used:_} = try_consume_declaration_modifiers(tokens_queue, &curr_queue_idx, local_variables, data_type)?;

    assert!(tokens_queue.peek(&curr_queue_idx) != Some(Token::PUNCTUATOR(Punctuator::OPENCURLY)), "found a function, and I can't handle that yet");

    let data_type = DataType{
        type_info: data_type.clone(),
        modifiers: modifiers.modifiers,
    };

    let extra_stack_needed = data_type.memory_size();//get the size of this variable

    let decl = Declaration {
        name: var_name.to_string(),
        data_type
    };

    local_variables.add_variable(decl.clone());//save variable to variable list early, so that I can reference it in the initialisation

    curr_queue_idx = remaining_tokens;//tokens have been consumed

    //try to match an initialisation expression
    let initialisation = consume_initialisation(tokens_queue, &mut curr_queue_idx, local_variables, &var_name);

    Some(ASTMetadata {
        resultant_tree: InitialisedDeclaration {decl, initialisation}, 
        remaining_slice: TokenQueueSlice::empty(),
        extra_stack_used: extra_stack_needed
    })
}

/**
 * takes the *x[3] part of int *x[3] = {1,2,3};
 * and parses the modifiers in order
 * TODO function pointers not supported
 */
pub fn try_consume_declaration_modifiers(tokens_queue: &mut TokenQueue, slice: &TokenQueueSlice, local_variables: &mut StackVariables, data_type: &Vec<TypeInfo>) -> Option<ASTMetadata<Declaration>> {
    let mut curr_queue_idx = slice.clone();

    let mut pointer_modifiers = Vec::new();
    let mut array_modifiers = Vec::new();

    loop {
        if tokens_queue.peek(&curr_queue_idx).unwrap() == Token::PUNCTUATOR(Punctuator::ASTERISK) {
            tokens_queue.consume(&mut curr_queue_idx);//consume the token
            pointer_modifiers.push(DeclModifier::POINTER);
        } else {
            break;//no more pointer info
        }
    }

    //declarations are expected to go **(something)[][]
    //so detect whether something is in brackets, or just an identifier
    let inner_data = match tokens_queue.peek(&curr_queue_idx).unwrap() {
        Token::PUNCTUATOR(Punctuator::OPENCURLY) => {
            //find the corresponding close bracket, and deal with it
            let in_brackets_tokens = tokens_queue.consume_inside_parenthesis(&mut curr_queue_idx);

            let parsed_in_brackets = try_consume_declaration_modifiers(tokens_queue, &in_brackets_tokens, local_variables, data_type)?;

            //curr queue idx is already advanced from consuming the parenthesis

            parsed_in_brackets.resultant_tree//return the inside
            
        },
        Token::IDENTIFIER(ident) => {
            tokens_queue.consume(&mut curr_queue_idx);//consume token
            //identifier name in the middle, grab it
            Declaration {
                data_type: DataType {
                    type_info: data_type.clone(),
                    modifiers: Vec::new(),
                },
                name: ident.to_string(),
            }
        }
        _ => panic!("unknown token in the middle of a declaration")
    };

    loop {
        match tokens_queue.peek(&curr_queue_idx) {
            Some(Token::PUNCTUATOR(Punctuator::OPENSQUARE)) => {

                tokens_queue.consume(&mut curr_queue_idx)?;//consume the open bracket
                if let Token::NUMBER(arraysize) = tokens_queue.consume(&mut curr_queue_idx)? {
                    array_modifiers.push(DeclModifier::ARRAY(arraysize.as_usize()?));
                } else {
                    panic!("array size inference not supported!")//I can't predict the size of arrays yet, so char[] x = "hello world";does not work
                }
                tokens_queue.consume(&mut curr_queue_idx)?;//consume the close bracket
            },
            _ => {break;}
        }
    }

    //take the example int *(*foo)[10]
    let ordered_modifiers: Vec<DeclModifier> = //foo is a:
        inner_data.data_type.modifiers.into_iter()//pointer to
        .chain(pointer_modifiers.into_iter())//pointer to
        .chain(array_modifiers.into_iter())//array of 10 integers
        .collect();

    Some(ASTMetadata {
        remaining_slice: curr_queue_idx,
        resultant_tree: Declaration {
            data_type: DataType {
                type_info: data_type.clone(),
                modifiers: ordered_modifiers,
            },
            name: inner_data.name,//inner always contains a name
        },
        extra_stack_used: MemoryLayout::new()//not my job
    })
}

/**
 * this consumes the tokens = 3+1 in the declaration int x= 3+1;
 * curr_queue_idx is mutable as this consumes tokens for the calling function
 * var_name what the name of the variable we are assigning to is
 */
fn consume_initialisation(tokens_queue: &mut TokenQueue, curr_queue_idx: &mut TokenQueueSlice, local_variables: &StackVariables, var_name: &str) -> Option<Expression> {
    
    if tokens_queue.peek(&curr_queue_idx)? !=Token::PUNCTUATOR(Punctuator::EQUALS){
        return None;
    }

    tokens_queue.consume(curr_queue_idx).unwrap();//consume the equals sign

    //consume the right hand side of the initialisation
    //then create an assignment expression to write the value to the variable
    //this should also work for pointer intitialisation, as that sets the address of the pointer
    Some(Expression::BINARYEXPR(
        Box::new(Expression::STACKVAR(local_variables.get_variable(var_name).unwrap())),
        Punctuator::EQUALS,
        Box::new(Expression::try_consume_whole_expr(tokens_queue, &curr_queue_idx, local_variables)?)
    ))
}