use memory_size::MemoryLayout;

use crate::{asm_generation::{self, asm_comment, asm_line, LogicalRegister, RegisterName}, ast_metadata::ASTMetadata, binary_expression::BinaryExpression, compilation_state::functions::FunctionList, data_type::{base_type::BaseType, data_type::DataType, type_modifier::DeclModifier}, enum_definition::try_consume_enum_as_type, expression::{self, ExprNode}, lexer::{keywords::Keyword, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size, scope_data::ScopeData};
use std::fmt::Write;

pub struct InitialisedDeclaration{
    decl: Declaration,
    initialisation: Option<Box<dyn ExprNode>>,
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

impl ExprNode for AddressedDeclaration {
    fn generate_assembly(&self) -> String {
        let mut result = String::new();

        if self.decl.data_type.is_array() {
            //getting an array, decays to a pointer
            asm_comment!(result, "decaying array {} to pointer", self.decl.name);
            asm_line!(result, "lea {}, [rbp-{}]", LogicalRegister::ACC.generate_reg_name(&asm_generation::PTR_SIZE), self.stack_offset.size_bytes());
        } else {
            let reg_size = &self.decl.data_type.memory_size();//decide which register size is appropriate for this variable
            asm_comment!(result, "reading variable: {} to register {}", self.decl.name, LogicalRegister::ACC.generate_reg_name(reg_size));

            asm_line!(result, "mov {}, [rbp-{}]", LogicalRegister::ACC.generate_reg_name(reg_size), self.stack_offset.size_bytes());//get the value from its address on the stack
        }

        result
    }

    fn get_data_type(&self) -> DataType {
        self.decl.get_type().clone()
    }
    
    fn put_lvalue_addr_in_acc(&self) -> String {
        let mut result = String::new();

        asm_comment!(result, "getting address of variable: {}", self.decl.name);
        asm_line!(result, "lea rax, [rbp-{}]", self.stack_offset.size_bytes());//calculate the address of the variable

        result
    }
}

impl InitialisedDeclaration {
    /**
     * local_variables is mut as variables are added
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, scope_data: &mut ScopeData) -> Option<ASTMetadata<Vec<InitialisedDeclaration>>> {
        let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        let mut declarations = Vec::new();
        let mut extra_stack_needed = MemoryLayout::new();
        
        //consume int or unsigned int or enum etc.
        let base_type = consume_base_type(tokens_queue, &mut curr_queue_idx, scope_data)?;

        //find semicolon
        let semicolon_idx = tokens_queue.find_closure_in_slice(&curr_queue_idx, false, |x| *x == Token::PUNCTUATOR(Punctuator::SEMICOLON))?;
        //find where all the declarators are (the x=2,y part in int x=2,y;)
        let all_declarators_segment = TokenQueueSlice{index:curr_queue_idx.index, max_index:semicolon_idx.index};
        //split each declarator
        let declarator_segments = tokens_queue.split_outside_parentheses(&all_declarators_segment, |x| *x == Token::PUNCTUATOR(Punctuator::COMMA));

        for declarator_segment in declarator_segments {
            //try and consume the declarator
            if let Some(ASTMetadata { remaining_slice: _, resultant_tree, extra_stack_used }) = try_consume_declarator(tokens_queue, &declarator_segment, &base_type, accessible_funcs, scope_data) {
                declarations.push(resultant_tree);//the declarator consumption actaully gives us a full declaration
                extra_stack_needed += extra_stack_used;
            }
        }

        curr_queue_idx = TokenQueueSlice{index: semicolon_idx.index + 1, max_index: curr_queue_idx.max_index};//consume the semicolon

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
pub fn try_consume_declarator(tokens_queue: &mut TokenQueue, slice: &TokenQueueSlice, base_type: &BaseType, accessible_funcs: &FunctionList, scope_data: &mut ScopeData) -> Option<ASTMetadata<InitialisedDeclaration>> {
    if slice.get_slice_size() == 0 {
        return None;//obviously no declarations in ""
    }
    let mut curr_queue_idx = slice.clone();

    //by parsing the *x[2] part of int *x[2];, I can get the modifiers and the variable name
    let ASTMetadata{resultant_tree: Declaration { data_type: modifiers, name: var_name }, remaining_slice:remaining_tokens, extra_stack_used:_} = try_consume_declaration_modifiers(tokens_queue, &curr_queue_idx, base_type)?;

    assert!(tokens_queue.peek(&curr_queue_idx) != Some(Token::PUNCTUATOR(Punctuator::OPENCURLY)), "found a function, and I can't handle that yet");

    let data_type = DataType::new_from_base_type(&base_type, modifiers.get_modifiers());

    let extra_stack_needed = data_type.memory_size();//get the size of this variable

    let decl = Declaration {
        name: var_name.to_string(),
        data_type
    };

    scope_data.stack_vars.add_variable(decl.clone());//save variable to variable list early, so that I can reference it in the initialisation

    curr_queue_idx = remaining_tokens;//tokens have been consumed

    //try to match an initialisation expression
    let initialisation = consume_initialisation(tokens_queue, &mut curr_queue_idx, &var_name, accessible_funcs, scope_data);

    Some(ASTMetadata {
        resultant_tree: InitialisedDeclaration {decl, initialisation}, 
        remaining_slice: TokenQueueSlice::empty(),
        extra_stack_used: extra_stack_needed
    })
}

/**
 * takes the *x[3] part of int *x[3] = {1,2,3};
 * and parses the modifiers in order
 * also used in function params
 * TODO function pointers not supported
 */
pub fn try_consume_declaration_modifiers(tokens_queue: &mut TokenQueue, slice: &TokenQueueSlice, base_type: &BaseType) -> Option<ASTMetadata<Declaration>> {
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

            let parsed_in_brackets = try_consume_declaration_modifiers(tokens_queue, &in_brackets_tokens, base_type)?;

            //curr queue idx is already advanced from consuming the parenthesis

            parsed_in_brackets.resultant_tree//return the inside
            
        },
        Token::IDENTIFIER(ident) => {
            tokens_queue.consume(&mut curr_queue_idx);//consume token
            //identifier name in the middle, grab it
            Declaration {
                data_type: DataType::new_from_base_type(&base_type, &Vec::new()),
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
        [inner_data.data_type.get_modifiers(),//pointer to
         &pointer_modifiers, //pointer to
         &array_modifiers].concat();//array of 10 integers

    Some(ASTMetadata {
        remaining_slice: curr_queue_idx,
        resultant_tree: Declaration {
            data_type: DataType::new_from_base_type(&base_type, &ordered_modifiers),
            name: inner_data.name,//inner always contains a name
        },
        extra_stack_used: MemoryLayout::new()//not my job
    })
}

pub fn consume_base_type(tokens_queue: &mut TokenQueue, curr_queue_idx: &mut TokenQueueSlice, scope_data: &mut ScopeData) -> Option<BaseType> {

    if tokens_queue.peek(&curr_queue_idx)? == Token::KEYWORD(Keyword::ENUM) {
        //enum x => handle enums
        if let Some(data_type) = try_consume_enum_as_type(tokens_queue, curr_queue_idx, scope_data) {
            return Some(data_type.underlying_type().clone())
        }
    }

    //fallback to default type system

    let mut data_type_info = Vec::new();
    //try and consume as many type specifiers as possible
    loop {
        if let Token::TYPESPECIFIER(ts) = tokens_queue.peek(&curr_queue_idx)? {
            data_type_info.push(ts.clone());
            tokens_queue.consume(curr_queue_idx);
        } else {
            break;
        }
    }
    //create data type out of it, but just get the base type as it can never be a pointer/array etc.
    return Some(DataType::new_from_type_list(&data_type_info, &Vec::new()).underlying_type().clone())
}

/**
 * this consumes the tokens = 3+1 in the declaration int x= 3+1;
 * curr_queue_idx is mutable as this consumes tokens for the calling function
 * var_name what the name of the variable we are assigning to is
 */
fn consume_initialisation(tokens_queue: &mut TokenQueue, curr_queue_idx: &mut TokenQueueSlice, var_name: &str, accessible_funcs: &FunctionList, scope_data: &mut ScopeData) -> Option<Box<dyn ExprNode>> {
    
    if tokens_queue.peek(&curr_queue_idx)? !=Token::PUNCTUATOR(Punctuator::EQUALS){
        return None;
    }

    tokens_queue.consume(curr_queue_idx).unwrap();//consume the equals sign

    //consume the right hand side of the initialisation
    //then create an assignment expression to write the value to the variable
    //this should also work for pointer intitialisation, as that sets the address of the pointer
    Some(Box::new(BinaryExpression::new(
        Box::new(scope_data.stack_vars.get_variable(var_name).unwrap()),
        Punctuator::EQUALS,
        expression::try_consume_whole_expr(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data).unwrap()
    )))
}