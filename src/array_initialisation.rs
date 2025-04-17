use crate::{compilation_state::functions::FunctionList, data_type::recursive_data_type::DataType, expression::{self, Expression}, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, number_literal::NumberLiteral, parse_data::ParseData};

#[derive(Clone, Debug)]
pub struct ArrayInitialisation {
    elements: Vec<Expression>
}

impl ArrayInitialisation {
    /// parses initialisation like {1, 2, 3}
    pub fn try_consume_whole_expr(tokens_queue: &TokenQueue, previous_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, scope_data: &mut ParseData) -> Option<Self> {
        if !tokens_queue.slice_is_brackets(previous_queue_idx, Punctuator::OPENSQUIGGLY) {
            return None;//initialisation must be the whole slice
        }

        //strip the { }
        let curr_queue_idx = TokenQueueSlice {
            index: previous_queue_idx.index + 1,
            max_index: previous_queue_idx.max_index - 1,
        };

        let items = tokens_queue.split_outside_parentheses(&curr_queue_idx, |x| *x == Token::PUNCTUATOR(Punctuator::COMMA), &TokenSearchType::skip_all());

        let mut parsed = Vec::new();

        for slice in items {
            //try to convert each slice to an expression
            parsed.push(
                expression::try_consume_whole_expr(tokens_queue, &slice, accessible_funcs, scope_data)?//return None early if any slice is not an expression
            );
        }

        Some(
            ArrayInitialisation{ elements: parsed }
        )
    }

    /**
     * flattens nested initialisation
     * for example: {{1},2,3,{4,5}} => {1,2,3,4,5}
     */
    /*pub fn flatten(self) -> Self {
        let new_elements = self.elements.into_iter()//get my elements
        //if the element is an array literal, flatten it, then merge an iterator of its elements
        //if the element is not, add just itself as an iterator
        .map(|i| {
            if let Expression::ARRAYLITERAL(arr) = i {
                arr.flatten().elements.into_iter()
            } else {
                vec![i].into_iter()
            }
        })
        //flatten into a single iterator
        .flatten()
        //collect to list
        .collect();

        Self { elements: new_elements }
    }*/

    /**
     * zero fills and rearranges elements to fit the structure of
     */
    /*pub fn conform_to_type(self, data_type: &DataType) -> Self {
        let zero_element = ArrayInitialisation::NUMBER(Box::new(Expression::NUMBERLITERAL(NumberLiteral::new("0"))));
        match self {
            ArrayInitialisation::ARRAY { elements } => {
                //array initialistion {1,2,3} must always define an array?
                unwrap_let!(DataType::ARRAY { size, element: arr_element_type } = data_type);

                let extra_zeros_required: usize = *size as usize - elements.len();

                //int x[2][2] = {{1}, 2, 3} => {{1,0}, {2,3}}
                let casted_zero_extended_elements: Vec<ArrayInitialisation> = 
                    elements.into_iter()//get all current elements
                    .chain(std::iter::repeat_n(zero_element, extra_zeros_required))//pad with zeros to correct array length required
                    .map(|x| x.conform_to_type(&arr_element_type))//convert each element to correct type
                    .collect();

                Self::ARRAY { elements: casted_zero_extended_elements }
            },
            ArrayInitialisation::NUMBER(expression) => {
                match data_type {
                    DataType::ARRAY { size, element } => {

                    },
                    _ => self //number can already be trivially cast to pointer or other numerical data type
                }
            },
        }
    }*/

    pub fn zero_fill_and_flatten_to_iter(&self, data_type: &DataType) -> Vec<Expression> {
        match data_type {
            DataType::ARRAY {element: element_type, .. } => {

                //apply recursively to current elements
                let flattened_elements: Vec<_> = self.elements.iter()
                    //int x[2][2] = {{1}, 2,3} => { {2,0}, {2}, {5} }
                    .map(|x| {
                        if let Expression::ARRAYLITERAL(arr) = x {
                            arr.zero_fill_and_flatten_to_iter(element_type)
                        } else {
                            vec![x.clone()]
                        }
                    })
                    //{ {2,0}, {2}, {5} } => {2,0,2,5}
                    .flatten()
                    .collect();

                let array_required_total_elements = data_type.array_num_elements();//int x[2][2] counts as 4 elements, since the array has already been flattened
                //pad with extra zeroes to fill to required size
                let extra_zeroes_required = array_required_total_elements as usize - flattened_elements.len();
                let zero_padded = flattened_elements.into_iter()
                    // int x[4] = {1,2} => {1,2,0,0}
                    .chain(std::iter::repeat_n(
                        Expression::NUMBERLITERAL(NumberLiteral::new("0")), extra_zeroes_required)
                    )
                    .collect();
                
                zero_padded
            }

            _ => panic!("tried to convert array initialisation to non-array?")

        }
    }
}