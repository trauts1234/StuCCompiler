use crate::{compilation_state::functions::FunctionList, data_type::recursive_data_type::DataType, debugging::{ASTDisplay, TreeDisplayInfo}, expression::expression::{self, Expression}, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, number_literal::typed_value::NumberLiteral, parse_data::ParseData};

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

        if curr_queue_idx.get_slice_size() == 0 {
            //c23 empty array initialise
            //int x[4] = {};
            return Some(Self {
                elements: Vec::new(),
            });
        }

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

    pub fn calculate_element_count(&self) -> u64 {
        self.elements.iter()
        .map(|x| {
            if let Expression::ARRAYLITERAL(arr) = x {
                arr.calculate_element_count()//nested array literal, count its item count
            } else {
                1//not an array, just one element
            }
        })
        .sum()//calculate the sum of elements
    }

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
                        Expression::NUMBERLITERAL(NumberLiteral::from(0)), extra_zeroes_required)
                    )
                    .collect();
                
                zero_padded
            }

            _ => panic!("tried to convert array initialisation to non-array?")

        }
    }
}

impl ASTDisplay for ArrayInitialisation {
    fn display_ast(&self, f: &mut TreeDisplayInfo) {
        f.write("{ ");
        for element in &self.elements {
            element.display_ast(f);
        }
        f.write(" }");
    }
}