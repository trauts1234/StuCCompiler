use crate::{asm_gen_data::AsmData, assembly::assembly::Assembly, ast_metadata::ASTMetadata, binary_expression::BinaryExpression, compilation_state::functions::FunctionList, data_type::{base_type::{self, BaseType}, recursive_data_type::RecursiveDataType, type_modifier::DeclModifier}, enum_definition::try_consume_enum_as_type, expression::{self, Expression}, expression_visitors::{expr_visitor::ExprVisitor, put_scalar_in_acc::ScalarInAccVisitor}, lexer::{keywords::Keyword, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, memory_size::MemoryLayout, parse_data::ParseData, struct_definition::StructDefinition};

/**
 * stores a variable and assembly to construct it
 */
pub struct InitialisedDeclaration{
    init_code: Option<Expression>,
}


#[derive(Clone)]
pub struct MinimalDataVariable {
    pub(crate) name: String
}

impl MinimalDataVariable {
    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_variable(self)
    }
}

/**
 * stores enough data to declare a variable
 * name and data type
 */
#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub(crate) data_type: RecursiveDataType,
    pub(crate) name: String,
}

//TODO move this to it's own file
impl InitialisedDeclaration {
    /**
     * local_variables is mut as variables are added
     * consumes declarations/definitions of stack variables
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, scope_data: &mut ParseData) -> Option<ASTMetadata<Vec<InitialisedDeclaration>>> {

        let mut declarations = Vec::new();
        
        //consume int or unsigned int or enum etc.
        let ASTMetadata { remaining_slice, resultant_tree: data_type } = consume_base_type(tokens_queue, &previous_queue_idx, scope_data)?;

        let mut curr_queue_idx = remaining_slice.clone();

        //find semicolon
        let semicolon_idx = tokens_queue.find_closure_matches(&curr_queue_idx, false, |x| *x == Token::PUNCTUATOR(Punctuator::SEMICOLON), &TokenSearchType::skip_nothing())?;
        //find where all the declarators are (the x=2,y part in int x=2,y;)
        let all_declarators_segment = TokenQueueSlice{index:curr_queue_idx.index, max_index:semicolon_idx.index};
        //split each declarator
        let declarator_segments = tokens_queue.split_outside_parentheses(&all_declarators_segment, |x| *x == Token::PUNCTUATOR(Punctuator::COMMA));

        for declarator_segment in declarator_segments {
            //try and consume the declarator
            if let Some(ASTMetadata { remaining_slice: _, resultant_tree}) = try_consume_declarator(tokens_queue, &declarator_segment, &data_type, accessible_funcs, scope_data) {
                declarations.push(resultant_tree);//the declarator consumption actaully gives us a full declaration
            }
        }

        curr_queue_idx = TokenQueueSlice{index: semicolon_idx.index + 1, max_index: curr_queue_idx.max_index};//consume the semicolon

        Some(ASTMetadata {
            resultant_tree: declarations,
            remaining_slice: curr_queue_idx,
        })
    }

    pub fn generate_assembly(&self, asm_data: &AsmData, stack_data: &mut MemoryLayout) -> Assembly {
        let mut result = String::new();

        if let Some(init) = &self.init_code {
            let init_asm = init.accept(&mut ScalarInAccVisitor {asm_data, stack_data});
            asm_line!(result, "{}", init_asm);//init is an expression that assigns to the variable, so no more work for me
        }

        result
    }
}

impl Declaration {
    pub fn get_type(&self) -> &RecursiveDataType {
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
pub fn try_consume_declarator(tokens_queue: &mut TokenQueue, slice: &TokenQueueSlice, base_type: &BaseType, accessible_funcs: &FunctionList, scope_data: &mut ParseData) -> Option<ASTMetadata<InitialisedDeclaration>> {
    if slice.get_slice_size() == 0 {
        return None;//obviously no declarations in ""
    }
    let mut curr_queue_idx = slice.clone();

    //by parsing the *x[2] part of int *x[2];, I can get the modifiers and the variable name
    let ASTMetadata{resultant_tree: Declaration { data_type: data_type_with_modifiers, name: var_name }, remaining_slice:remaining_tokens} = try_consume_declaration_modifiers(tokens_queue, &curr_queue_idx, base_type, scope_data)?;

    assert!(tokens_queue.peek(&curr_queue_idx, scope_data) != Some(Token::PUNCTUATOR(Punctuator::OPENCURLY)), "found a function, and I can't handle that yet");

    scope_data.add_variable(&var_name,data_type_with_modifiers);//save variable to variable list early, so that I can reference it in the initialisation

    curr_queue_idx = remaining_tokens;//tokens have been consumed

    //try to match an initialisation expression
    let initialisation = consume_initialisation(tokens_queue, &mut curr_queue_idx, &var_name, accessible_funcs, scope_data)
        .map(|x| Expression::BINARYEXPRESSION(x));//wrap as binary expression

    Some(ASTMetadata {
        resultant_tree: InitialisedDeclaration {init_code:initialisation}, 
        remaining_slice: TokenQueueSlice::empty(),
    })
}

/**
 * takes the *x[3] part of int *x[3] = {1,2,3};
 * and parses the modifiers in order
 * also used in function params
 * TODO function pointers not supported
 */
pub fn try_consume_declaration_modifiers(tokens_queue: &TokenQueue, slice: &TokenQueueSlice, base_type: &BaseType, scope_data: &mut ParseData) -> Option<ASTMetadata<Declaration>> {
    let mut curr_queue_idx = slice.clone();

    let mut pointer_modifiers = Vec::new();
    let mut array_modifiers = Vec::new();

    loop {
        if tokens_queue.peek(&curr_queue_idx, scope_data).unwrap() == Token::PUNCTUATOR(Punctuator::ASTERISK) {
            tokens_queue.consume(&mut curr_queue_idx, &scope_data);//consume the token
            pointer_modifiers.push(DeclModifier::POINTER);
        } else {
            break;//no more pointer info
        }
    }

    //declarations are expected to go **(something)[][]
    //so detect whether something is in brackets, or just an identifier
    let inner_data = match tokens_queue.peek(&curr_queue_idx, &scope_data).unwrap() {
        Token::PUNCTUATOR(Punctuator::OPENCURLY) => {
            //find the corresponding close bracket, and deal with it
            let in_brackets_tokens = tokens_queue.consume_inside_parenthesis(&mut curr_queue_idx);

            let parsed_in_brackets = try_consume_declaration_modifiers(tokens_queue, &in_brackets_tokens, base_type, scope_data)?;

            //curr queue idx is already advanced from consuming the parenthesis

            parsed_in_brackets.resultant_tree//return the inside
            
        },
        Token::IDENTIFIER(ident) => {
            tokens_queue.consume(&mut curr_queue_idx, &scope_data);//consume token
            //identifier name in the middle, grab it
            Declaration {
                data_type: RecursiveDataType::RAW(base_type.clone()),
                name: ident.to_string(),
            }
        }
        x => panic!("unknown token in the middle of a declaration: {:?}", x)
    };

    loop {
        match tokens_queue.peek(&curr_queue_idx, &scope_data) {
            Some(Token::PUNCTUATOR(Punctuator::OPENSQUARE)) => {

                tokens_queue.consume(&mut curr_queue_idx, &scope_data)?;//consume the open bracket
                if let Token::NUMBER(arraysize) = tokens_queue.consume(&mut curr_queue_idx, &scope_data)? {
                    array_modifiers.push(DeclModifier::ARRAY(arraysize.as_usize()));
                } else {
                    panic!("array size inference not supported!")//I can't predict the size of arrays yet, so char[] x = "hello world";does not work
                }
                tokens_queue.consume(&mut curr_queue_idx, &scope_data)?;//consume the close bracket
            },
            _ => {break;}
        }
    }

    //iterator item 0 is the outermost modifier. if it was pointer, it would be a pointer to whatever the rest was
    let extra_modifiers = 
    pointer_modifiers.iter()//all pointers take priority
    .chain(array_modifiers.iter())//first on this iterator is the first [x] found after the variable name
    .cloned();
    
    let result_type = Declaration {
        data_type: 
            extra_modifiers
            .rev()//reverse, to put innermost first, then outer ones
            .fold(
                inner_data.get_type().clone(),//start with inner type
                |curr_type, modifier| curr_type.add_outer_modifier(modifier)//add each modifier, innermost first
            ),
        name: inner_data.get_name().to_string(),
    };

    Some(ASTMetadata {
        remaining_slice: curr_queue_idx,
        resultant_tree: result_type,
    })
}

pub fn consume_base_type(tokens_queue: &TokenQueue, previous_slice: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<ASTMetadata<BaseType>> {

    let mut curr_queue_idx = previous_slice.clone();

    match tokens_queue.peek(&curr_queue_idx, &scope_data)? {
        Token::KEYWORD(Keyword::ENUM) => {
            let ASTMetadata { remaining_slice, resultant_tree } = try_consume_enum_as_type(tokens_queue, &mut curr_queue_idx, scope_data).unwrap();

            Some(ASTMetadata { remaining_slice, resultant_tree })
        }
        Token::KEYWORD(Keyword::STRUCT) => {
            let ASTMetadata { remaining_slice, resultant_tree: struct_type } = StructDefinition::try_consume_struct_as_type(tokens_queue, &mut curr_queue_idx, scope_data).unwrap();

            Some(ASTMetadata {
                remaining_slice,
                resultant_tree: BaseType::STRUCT(struct_type.name.clone().expect("not implemented: anonymous structs"))
            })
        }
        _ => {
            //fallback to default type system

            let mut data_type_info = Vec::new();
            //try and consume as many type specifiers as possible
            loop {
                if let Token::TYPESPECIFIER(ts) = tokens_queue.peek(&curr_queue_idx, &scope_data)? {
                    data_type_info.push(ts.clone());
                    tokens_queue.consume(&mut curr_queue_idx, &scope_data);
                } else {
                    break;
                }
            }

            Some(ASTMetadata {
                remaining_slice: curr_queue_idx,
                //create data type out of it, but just get the base type as it can never be a pointer/array etc.
                resultant_tree: base_type::new_from_type_list(&data_type_info)?
            })
        }
    }
}

/**
 * this consumes the tokens = 3+1 in the declaration int x= 3+1;
 * curr_queue_idx is mutable as this consumes tokens for the calling function
 * var_name what the name of the variable we are assigning to is
 * returns a binary expression assigning the new variable to its initial value
 */
fn consume_initialisation(tokens_queue: &mut TokenQueue, curr_queue_idx: &mut TokenQueueSlice, var_name: &str, accessible_funcs: &FunctionList, scope_data: &mut ParseData) -> Option<BinaryExpression> {
    
    if tokens_queue.peek(&curr_queue_idx, &scope_data)? !=Token::PUNCTUATOR(Punctuator::EQUALS){
        return None;
    }

    tokens_queue.consume(curr_queue_idx, &scope_data).unwrap();//consume the equals sign

    assert!(scope_data.variable_defined(var_name));

    //consume the right hand side of the initialisation
    //then create an assignment expression to write the value to the variable
    //this should also work for pointer intitialisation, as that sets the address of the pointer
    Some(BinaryExpression::new(
        Expression::VARIABLE(MinimalDataVariable{name: var_name.to_string()}),
        Punctuator::EQUALS,
        expression::try_consume_whole_expr(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data).unwrap()
    ))
}