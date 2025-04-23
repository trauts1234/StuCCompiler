use crate::{asm_gen_data::AsmData, assembly::assembly::Assembly, ast_metadata::ASTMetadata, binary_expression::BinaryExpression, compilation_state::functions::FunctionList, data_type::{base_type::{self, BaseType}, recursive_data_type::DataType, storage_type::StorageDuration, type_modifier::DeclModifier, type_token::TypeInfo}, debugging::ASTDisplay, declaration::{Declaration, MinimalDataVariable}, enum_definition::try_consume_enum_as_type,expression::{binary_expression_operator::BinaryExpressionOperator, expression::{self, Expression}}, expression_visitors::put_scalar_in_acc::ScalarInAccVisitor, lexer::{keywords::Keyword, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, parse_data::ParseData, struct_definition::StructDefinition};
use memory_size::MemorySize;

/**
 * stores a variable and assembly to construct it
 */
pub struct InitialisedDeclaration{
    init_code: Option<Expression>,
}

impl InitialisedDeclaration {
    /**
     * local_variables is mut as variables are added
     * consumes declarations/definitions of stack variables
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, scope_data: &mut ParseData) -> Option<ASTMetadata<Vec<InitialisedDeclaration>>> {

        let mut declarations = Vec::new();
        
        //consume int or unsigned int or enum etc.
        let ASTMetadata { remaining_slice, resultant_tree: (data_type, storage_duration) } = consume_type_specifier(tokens_queue, &previous_queue_idx, scope_data)?;

        let mut curr_queue_idx = remaining_slice.clone();

        //find semicolon
        let semicolon_idx = tokens_queue.find_closure_matches(&curr_queue_idx, false, |x| *x == Token::PUNCTUATOR(Punctuator::SEMICOLON), &TokenSearchType::skip_all())?;
        //find where all the declarators are (the x=2,y part in int x=2,y;)
        let all_declarators_segment = TokenQueueSlice{index:curr_queue_idx.index, max_index:semicolon_idx.index};
        //split each declarator
        let declarator_segments = tokens_queue.split_outside_parentheses(&all_declarators_segment, |x| *x == Token::PUNCTUATOR(Punctuator::COMMA), &TokenSearchType::skip_all());

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

    pub fn generate_assembly(&self, asm_data: &AsmData, stack_data: &mut MemorySize) -> Assembly {
        let mut result = Assembly::make_empty();

        if let Some(init) = &self.init_code {
            let init_asm = init.accept(&mut ScalarInAccVisitor {asm_data, stack_data});
            result.merge(&init_asm);//init is an expression that assigns to the variable, so no more work for me
        }

        result
    }
}

impl ASTDisplay for InitialisedDeclaration {
    fn display_ast(&self, f: &mut crate::debugging::TreeDisplayInfo) {
        if let Some(init) = &self.init_code {
            init.display_ast(f);
        }
    }
}

/**
 * claims to consume a declarator, but actaully takes in the data type too, and gives back a full declaration
 */
pub fn try_consume_declarator(tokens_queue: &mut TokenQueue, slice: &TokenQueueSlice, base_type: &DataType, accessible_funcs: &FunctionList, scope_data: &mut ParseData) -> Option<ASTMetadata<InitialisedDeclaration>> {
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
 * function pointers not supported
 */
pub fn try_consume_declaration_modifiers(tokens_queue: &TokenQueue, slice: &TokenQueueSlice, base_type: &DataType, scope_data: &ParseData) -> Option<ASTMetadata<Declaration>> {
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
                data_type: base_type.clone(),
                name: ident.to_string(),
            }
        }
        x => panic!("unknown token in the middle of a declaration: {:?}", x)
    };

    loop {
        match tokens_queue.peek(&curr_queue_idx, &scope_data) {
            Some(Token::PUNCTUATOR(Punctuator::OPENSQUARE)) => {

                tokens_queue.consume(&mut curr_queue_idx, &scope_data)?;//consume the open bracket
                match tokens_queue.consume(&mut curr_queue_idx, scope_data).unwrap() {
                    Token::NUMBER(arraysize) => {
                        array_modifiers.push(DeclModifier::ARRAY(arraysize.get_value().clone().try_into().unwrap()));
                        assert_eq!(tokens_queue.consume(&mut curr_queue_idx, &scope_data)?, Token::PUNCTUATOR(Punctuator::CLOSESQUARE));//consume the close bracket
                    }
                    Token::PUNCTUATOR(Punctuator::CLOSESQUARE) => {
                        array_modifiers.push(DeclModifier::UnknownSizeArray);
                        //close bracket already consumed
                    },

                    _ => panic!("unknown token in array declaration")
                }
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

/// this stores the partially calculated data type for consume_base_type
enum DataTypeInfo {
    Partial(Vec<TypeInfo>),//when collecting "int" "unsigned" etc.
    Full(DataType),//when a complete data type has been found, i.e enum/typedef
}

pub struct ConsumedBaseType {
    data_type: DataTypeInfo,
    storage_duration: Option<StorageDuration>
}

impl ConsumedBaseType {
    pub fn new() -> Self {
        ConsumedBaseType {
            data_type: DataTypeInfo::Partial(Vec::new()),
            storage_duration: Some(StorageDuration::Automatic),//TODO implement static and extern, leave this as None
        }
    }
    ///calculates and returns the data type and storage duration, consuming the ConsumedBaseType
    pub fn type_and_duration(self) -> Option<(DataType, StorageDuration)> {
        let complete_data_type = match self.data_type {
            DataTypeInfo::Partial(x) if x.len() == 0 => return None,//no information on type, must be a failure
            DataTypeInfo::Partial(type_infos) => DataType::RAW(base_type::new_from_type_list(&type_infos)),
            DataTypeInfo::Full(data_type) => data_type,
        };

        Some((complete_data_type, self.storage_duration?))
    }

    fn add_type_info(&mut self, extra: TypeInfo) {
        match &mut self.data_type {
            DataTypeInfo::Partial(type_infos) => type_infos.push(extra),
            DataTypeInfo::Full(_) => panic!("tried to add type info {:?} to a complete type", extra),
        }
    }
    fn add_complete_type(&mut self, new_type: DataType) {
        match &self.data_type {
            DataTypeInfo::Partial(type_infos) => assert!(type_infos.len() == 0),//perhaps I collected an "int" type and the whole type is being overwritten?
            DataTypeInfo::Full(_) => panic!("tried to overwrite completed type"),//perhaps I had a struct x, then tried to overwrite with struct y in "struct x struct y" or similar bad code
        }
        //overwrite my type
        self.data_type = DataTypeInfo::Full(new_type)
    }
}

pub fn consume_type_specifier(tokens_queue: &TokenQueue, queue_idx: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<ASTMetadata<(DataType, StorageDuration)>> {
    let ASTMetadata { remaining_slice, resultant_tree } = consume_type_specifier_recursive(tokens_queue, queue_idx, scope_data, ConsumedBaseType::new());

    Some(ASTMetadata {
        remaining_slice,
        resultant_tree: resultant_tree.type_and_duration()?,//try and get data, or fail
    })
}

/// a recursive function that consumes the "unsigned int" or "struct x" part of a declaration
/// assumes that the queue starts with a valid type specifier
pub fn consume_type_specifier_recursive(tokens_queue: &TokenQueue, queue_idx: &TokenQueueSlice, scope_data: &mut ParseData, mut initial_type: ConsumedBaseType) -> ASTMetadata<ConsumedBaseType> {
    match tokens_queue.peek(queue_idx, &scope_data) {
        Some(Token::TYPESPECIFIER(ts)) => {
            initial_type.add_type_info(ts);
            //recursively get other type specifiers
            consume_type_specifier_recursive(tokens_queue, &queue_idx.next_clone(), scope_data, initial_type)
        }

        Some(Token::KEYWORD(Keyword::ENUM)) => {
            let ASTMetadata { remaining_slice, resultant_tree } = try_consume_enum_as_type(tokens_queue, &mut queue_idx.clone(), scope_data).unwrap();

            initial_type.add_complete_type(DataType::RAW(resultant_tree));//enum specifies a type, so no need for "int" and "unsigned" etc.

            consume_type_specifier_recursive(tokens_queue, &remaining_slice, scope_data, initial_type)
        }
        Some(Token::KEYWORD(Keyword::STRUCT)) => {
            let ASTMetadata { remaining_slice, resultant_tree: struct_name } = StructDefinition::try_consume_struct_as_type(tokens_queue, &mut queue_idx.clone(), scope_data).unwrap();

            initial_type.add_complete_type(DataType::RAW(BaseType::STRUCT(struct_name)));//struct specifies a whole type so just store that

            consume_type_specifier_recursive(tokens_queue, &remaining_slice, scope_data, initial_type)//recursively look for more info
        }
        Some(Token::IDENTIFIER(name)) => {
            match scope_data.get_typedef(&name) {
                Some(x) => {
                    initial_type.add_complete_type(x.clone());//get type of typedef and fill it in
                    consume_type_specifier_recursive(tokens_queue, &queue_idx.next_clone(), scope_data, initial_type)//consume other info
                }
                None => ASTMetadata { remaining_slice: queue_idx.clone(), resultant_tree: initial_type }//unknown identifier, probably a variable name
            }
        },

        _ => ASTMetadata { remaining_slice: queue_idx.clone(), resultant_tree: initial_type } //invalid, return what has already been parsed
    }
}

/**
 * this consumes the tokens = 3+1 in the declaration int x= 3+1;
 * curr_queue_idx is mutable as this consumes tokens for the calling function
 * var_name what the name of the variable we are assigning to is
 * returns a binary expression assigning the new variable to its initial value
 */
fn consume_initialisation(tokens_queue: &mut TokenQueue, curr_queue_idx: &mut TokenQueueSlice, var_name: &str, accessible_funcs: &FunctionList, scope_data: &mut ParseData) -> Option<BinaryExpression> {
    
    if tokens_queue.peek(&curr_queue_idx, &scope_data)? != Token::PUNCTUATOR(Punctuator::EQUALS){
        return None;
    }

    tokens_queue.consume(curr_queue_idx, &scope_data).unwrap();//consume the equals sign

    assert!(scope_data.variable_defined(var_name));

    //consume the right hand side of the initialisation
    //then create an assignment expression to write the value to the variable
    //this should also work for pointer intitialisation, as that sets the address of the pointer
    Some(BinaryExpression::new(
        Expression::VARIABLE(MinimalDataVariable{name: var_name.to_string()}),
        BinaryExpressionOperator::Assign,
        expression::try_consume_whole_expr(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data).unwrap()
    ))
}