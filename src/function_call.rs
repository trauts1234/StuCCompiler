use crate::{asm_boilerplate, asm_generation::{self, asm_comment, asm_line, Register}, compilation_state::{functions::FunctionList, stack_variables::StackVariables}, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, type_info::DataType};
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct FunctionCall {
    //params TODO,
    func_name: String,//maybe an enum, for function pointers
    return_type: DataType//clone this from the function I am calling
}

impl FunctionCall {
    pub fn get_data_type(&self) -> DataType {
        self.return_type.clone()
    }
    pub fn try_consume_whole_expr(tokens_queue: &mut TokenQueue, curr_queue_idx: &TokenQueueSlice, _local_variables: &StackVariables, accessible_funcs: &FunctionList) -> Option<FunctionCall> {
        //look for unary postfixes as association is left to right
        let last_token = tokens_queue.peek_back(&curr_queue_idx)?;
    
        if last_token != Token::PUNCTUATOR(Punctuator::CLOSECURLY){
            return None;
        }
    
        let square_open_idx = tokens_queue.find_matching_open_bracket(curr_queue_idx.max_index-1);//-1 as max index is exclusive
    
        let params_slice = TokenQueueSlice {
            index: square_open_idx+1,
            max_index: curr_queue_idx.max_index-1
        };

        let func_slice = TokenQueueSlice {//array or pointer etc.
            index: curr_queue_idx.index,
            max_index: square_open_idx
        };

        if let Token::IDENTIFIER(func_name) = tokens_queue.peek(&func_slice)? {
            let func_definition = accessible_funcs.get_function(&func_name).expect("found function call but no corresponding function definition");
            Some(FunctionCall {func_name, return_type: func_definition.get_return_type()})
        } else {
            None
        }
    }

    /**
     * puts the return value on the stack
     */
    pub fn generate_assembly(&self) -> String {
        //system V ABI
        let mut result = String::new();

        let return_reg_name = asm_generation::generate_reg_name(&self.return_type.memory_size(), Register::ACC);//most integer and pointer args are returned in _AX register

        asm_comment!(result, "calling function: {}", self.func_name);
        asm_line!(result, "call {}", self.func_name);
        asm_line!(result, "{}", asm_boilerplate::push_reg(&return_reg_name));//put return value on stack

        result
        
    }
}