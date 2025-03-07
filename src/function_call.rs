use crate::{asm_boilerplate, asm_generation::{self, asm_comment, asm_line, LogicalRegister}, compilation_state::{functions::FunctionList, stack_variables::StackVariables}, expression::Expression, function_declaration::FunctionDeclaration, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size::MemoryLayout, type_info::DataType};
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct FunctionCall {
    func_name: String,//maybe an enum, for function pointers
    args: Vec<Expression>,

    decl: FunctionDeclaration
}

impl FunctionCall {

    pub fn get_data_type(&self) -> DataType {
        self.decl.return_type.clone()
    }
    
    pub fn try_consume_whole_expr(tokens_queue: &mut TokenQueue, curr_queue_idx: &TokenQueueSlice, local_variables: &StackVariables, accessible_funcs: &FunctionList) -> Option<FunctionCall> {
        //look for unary postfixes as association is left to right
        let last_token = tokens_queue.peek_back(&curr_queue_idx)?;
    
        if last_token != Token::PUNCTUATOR(Punctuator::CLOSECURLY){
            return None;
        }
    
        let curly_open_idx = tokens_queue.find_matching_open_bracket(curr_queue_idx.max_index-1);//-1 as max index is exclusive
    
        let all_args_slice = TokenQueueSlice {
            index: curly_open_idx+1,
            max_index: curr_queue_idx.max_index-1
        };

        let args_slices = tokens_queue.split_outside_parentheses(&all_args_slice, |x| *x == Token::PUNCTUATOR(Punctuator::COMMA));

        let mut args = Vec::new();

        if all_args_slice.get_slice_size() > 0 {//ensure args have actually been passed
            for arg_slice in args_slices {
                args.push(Expression::try_consume_whole_expr(tokens_queue, &arg_slice, local_variables, accessible_funcs)?);
            }
        }

        let func_slice = TokenQueueSlice {//array or pointer etc.
            index: curr_queue_idx.index,
            max_index: curly_open_idx
        };

        if let Token::IDENTIFIER(func_name) = tokens_queue.peek(&func_slice)? {
            let func_decl = accessible_funcs.get_function_declaration(&func_name).expect("found function call but no corresponding function declaration");//for recursive functions this is fine, right?
            Some(FunctionCall {
                func_name, 
                args,
                decl: func_decl.clone(),
            })
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

        asm_comment!(result, "calling function: {}", self.func_name);

        //put args on the stack as 64 bits
        for (i, arg) in self.args.iter().enumerate().rev() {//go through each arg and param right to left
            let param_type = &self.decl.params[i.min(self.decl.params.len()-1)];//when len(params) > len(args), grab the last of params, as it could be a varadic param
            
            if i >= self.decl.params.len() {
                assert!(param_type.get_type().is_varadic_param());//more args than params, so must be varadic
            }

            asm_line!(result, "{}", arg.generate_assembly());//calculate the arg
            asm_line!(result, "{}", asm_boilerplate::cast_from_acc(&arg.get_data_type(), param_type.get_type()));//cast to requested type

            asm_line!(result, "{}", asm_boilerplate::push_reg(&MemoryLayout::from_bytes(8), &LogicalRegister::ACC));//implicitly extend to 8 bytes, without conversion/casting
        }

        //loop thru args that could be put in registers (any params up to 6)
        for i in 0..self.args.len().min(6) {//note no reversal because the stack is LIFO, so the last param(param on lhs) is the first here
            asm_line!(result, "{}", asm_boilerplate::pop_reg(&MemoryLayout::from_bytes(8), &asm_generation::generate_param_reg(i)));//store the param in the correct register
        }

        asm_line!(result, "call {}", self.func_name);

        if self.args.len() > 6 {
            //some args were put on the stack
            asm_line!(result, "add rsp, {} ;remove stack params", 8*(self.args.len()-6));
        }

        result
        
    }
}