use crate::{asm_gen_data::GetStruct, ast_metadata::ASTMetadata, compilation_state::label_generator::LabelGenerator, constexpr_parsing::ConstexprValue, data_type::{base_type::{BaseType, IntegerType, ScalarType}, recursive_data_type::DataType, storage_type::StorageDuration}, debugging::IRDisplay, declaration::Declaration, expression::expression::try_consume_whole_expr, initialised_declaration::{ consume_type_specifier, try_consume_declaration_modifiers}, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, number_literal::typed_value::NumberLiteral, parse_data::ParseData};


pub struct GlobalVariable {
    decl: Declaration,
    default_value: ConstexprValue,//perhaps some more abstract data type when structs are implemented
    storage_class: StorageDuration
}

impl GlobalVariable {
    pub fn generate_assembly(&self, struct_info: &dyn GetStruct) -> String {
        match (&self.decl.data_type, &self.default_value) {
            //base type is set to a value
            (DataType::RAW(BaseType::Scalar(base_type)), ConstexprValue::NUMBER(number_literal)) => {
                //cast the number to the variable's type, then generate a constant
                number_literal.cast(&base_type).generate_data_definition_instruction(&self.decl.name)
            },

            // array is being set to a string
            (DataType::ARRAY { size, element }, ConstexprValue::STRING(string_literal)) => {
                assert_eq!(**element, DataType::RAW(BaseType::Scalar(ScalarType::Integer(IntegerType::I8))));
                assert_eq!(*size as usize, string_literal.get_num_chars());
                format!("{} db {}\n", self.decl.name, string_literal.get_comma_separated_bytes())
            }

            // unknown size array is being set to a string
            (DataType::UNKNOWNSIZEARRAY { element }, ConstexprValue::STRING(string_literal)) => {
                assert_eq!(**element, DataType::RAW(BaseType::Scalar(ScalarType::Integer(IntegerType::I8))));
                format!("{} db {}\n", self.decl.name, string_literal.get_comma_separated_bytes())
            }

            // pointer is being set to string
            // so make the pointer point at the string
            (DataType::POINTER(element), ConstexprValue::STRING(string_literal)) => {
                assert_eq!(**element, DataType::RAW(BaseType::Scalar(ScalarType::Integer(IntegerType::I8))));
                format!("{} dq {}\n", self.decl.name, string_literal.get_label())
            }

            //  pointer being set to a pointer...
            (DataType::POINTER(_), ConstexprValue::POINTER { label, offset }) => {
                format!("{} dq {} + {}\n", self.decl.name, label,
                match offset {
                    NumberLiteral::INTEGER{data, ..} => data,
                    NumberLiteral::FLOAT {..} => panic!("cannot offset a constant pointer by a float")
                })
            }

            // zeroing out the variable
            (data_type, ConstexprValue::ZEROES) => format!("{} TIMES {} db 0", self.decl.name, data_type.memory_size(struct_info).size_bytes()),

            (t, val) => panic!("cannot initialise {:?} to {:?}", t, val)
        }
    }

    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData, struct_label_gen: &mut LabelGenerator) -> Option<ASTMetadata<Vec<GlobalVariable>>> {

        let mut declarations = Vec::new();
        
        //consume int or unsigned int or enum etc.
        let ASTMetadata { remaining_slice, resultant_tree: (base_type, storage_duration) } = consume_type_specifier(tokens_queue, previous_queue_idx, scope_data, struct_label_gen)?;

        let mut curr_queue_idx = remaining_slice.clone();

        //find semicolon
        let semicolon_idx = tokens_queue.find_closure_matches(&curr_queue_idx, false, |x| *x == Token::PUNCTUATOR(Punctuator::SEMICOLON), &TokenSearchType::skip_all_brackets())?;
        //find where all the declarators are (the x=2,y part in int x=2,y;)
        let all_declarators_segment = TokenQueueSlice{index:curr_queue_idx.index, max_index:semicolon_idx};
        //split each declarator
        let declarator_segments = tokens_queue.split_outside_parentheses(&all_declarators_segment, |x| *x == Token::PUNCTUATOR(Punctuator::COMMA), &TokenSearchType::skip_all_brackets());

        for declarator_segment in declarator_segments {
            //try and consume the declarator
            if let Some(ASTMetadata { resultant_tree, .. }) = try_consume_constexpr_declarator(tokens_queue, &declarator_segment, &base_type, storage_duration.clone(), scope_data, struct_label_gen) {
                declarations.push(resultant_tree);//the declarator consumption actaully gives us a full declaration
            }
        }

        curr_queue_idx = TokenQueueSlice{index: semicolon_idx + 1, max_index: curr_queue_idx.max_index};//consume the semicolon

        Some(ASTMetadata {
            resultant_tree: declarations,
            remaining_slice: curr_queue_idx,
        })
    }

    pub fn storage_class(&self) -> &StorageDuration {
        &self.storage_class
    }
    pub fn var_name(&self) -> &str {
        &self.decl.name
    }
}

impl IRDisplay for GlobalVariable {
    fn display_ir(&self) -> String {
        format!("global var {} = {} ({})", self.decl.name, self.default_value.display_ir(), self.decl.data_type)
    }
}

fn try_consume_constexpr_declarator(tokens_queue: &mut TokenQueue, slice: &TokenQueueSlice, base_type: &DataType, storage_class: StorageDuration, scope_data: &mut ParseData, struct_label_gen: &mut LabelGenerator) -> Option<ASTMetadata<GlobalVariable>> {
    if slice.get_slice_size() == 0 {
        return None;
    }
    
    let mut curr_queue_idx = slice.clone();
    
    let ASTMetadata{resultant_tree: Declaration { data_type, name: var_name }, remaining_slice:remaining_tokens} = try_consume_declaration_modifiers(tokens_queue, &curr_queue_idx, base_type, scope_data, struct_label_gen)?;

    scope_data.add_variable(&var_name, data_type.clone());//save variable to variable list early, so that I can reference it in the initialisation

    let decl = Declaration {
        name: var_name.to_string(),
        data_type
    };

    curr_queue_idx = remaining_tokens;//tokens have been consumed

    //try to match an initialisation expression
    let default_value = consume_constexpr_initialisation(tokens_queue, &mut curr_queue_idx, scope_data, struct_label_gen);

    Some(ASTMetadata {
        resultant_tree: GlobalVariable {
            decl,
            default_value,
            storage_class
        }, 
        remaining_slice: TokenQueueSlice::empty(),
    })
}

fn consume_constexpr_initialisation(tokens_queue: &mut TokenQueue, curr_queue_idx: &mut TokenQueueSlice, scope_data: &mut ParseData, struct_label_gen: &mut LabelGenerator) -> ConstexprValue {
    if tokens_queue.peek(&curr_queue_idx, &scope_data) != Some(Token::PUNCTUATOR(Punctuator::EQUALS)){
        return ConstexprValue::ZEROES;
    }

    tokens_queue.consume(curr_queue_idx, &scope_data).unwrap();//consume the equals sign

    //pass empty function list as it should never call functions anyways
    try_consume_whole_expr(tokens_queue, curr_queue_idx, scope_data, struct_label_gen)//return the consumed value for the variable
    .map(|x| (&x).try_into().unwrap()) // fold to constant
    .expect(&tokens_queue.display_slice(curr_queue_idx))
}