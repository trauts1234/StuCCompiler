use crate::{asm_boilerplate, asm_generation::{self, asm_comment, asm_line, Register}, compilation_state::{functions::FunctionList, stack_variables::StackVariables}, expression::Expression, function_declaration::FunctionDeclaration, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size::MemoryLayout, type_info::DataType};
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct FunctionCall {
    func_name: String,//maybe an enum, for function pointers
    args: Vec<Expression>,

    extra_stack_for_alignment: MemoryLayout,//how much extra stack is reserved to allign the call

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

        for arg_slice in args_slices {
            args.push(Expression::try_consume_whole_expr(tokens_queue, &arg_slice, local_variables, accessible_funcs)?);
        }

        let func_slice = TokenQueueSlice {//array or pointer etc.
            index: curr_queue_idx.index,
            max_index: curly_open_idx
        };

        if let Token::IDENTIFIER(func_name) = tokens_queue.peek(&func_slice)? {
            let func_decl = accessible_funcs.get_function(&func_name).expect("found function call but no corresponding function definition");//for recursive functions this is fine, right?
            let stack_height_bytes = local_variables.get_stack_used().size_bytes() + 8*(args.len()-6).max(0);//first 6 args in registers, then on stack in 8 byte chunks
            let wanted_extra_stack = MemoryLayout::from_bytes(
                stack_height_bytes - (stack_height_bytes/16) * 16//finds the number of extra bytes needed to round to a 16 byte boundary
            );
            Some(FunctionCall {
                func_name, 
                args,
                decl: func_decl,
                extra_stack_for_alignment: wanted_extra_stack
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

        let return_reg_name = asm_generation::generate_reg_name(&self.decl.return_type.memory_size(), Register::ACC);//most integer and pointer args are returned in _AX register

        asm_comment!(result, "calling function: {}", self.func_name);

        asm_line!(result, "sub rsp, {} ;align the stack", self.extra_stack_for_alignment.size_bytes());

        for (i, (arg, param_type)) in self.args.iter().zip(self.decl.params.iter()).enumerate().rev() {//go through each arg and param right to left
            asm_line!(result, "{}", arg.generate_assembly());//calculate the arg
            asm_line!(result, "{}", asm_boilerplate::cast_from_stack(&arg.get_data_type(), param_type.get_type()));//cast to requested type

            let arg_temp_reg = asm_generation::generate_reg_name(&param_type.get_type().memory_size(), Register::ACC);
            asm_line!(result, "{}", asm_boilerplate::pop_reg(&arg_temp_reg));//pop to accumulator
            let arg_as_8byte = asm_generation::generate_reg_name(&MemoryLayout::from_bytes(8), Register::ACC);
            asm_line!(result, "{}", asm_boilerplate::pop_reg(&arg_as_8byte));//extend to 8 bytes, without conversion/casting

            //put the 64 bit into the right place
            match i {
                0 => {
                    asm_line!(result, "{}", asm_boilerplate::pop_reg("rdi"))//arg 0 goes in rdi
                },
                1 => {
                    asm_line!(result, "{}", asm_boilerplate::pop_reg("rsi"))
                },
                2 => {
                    asm_line!(result, "{}", asm_boilerplate::pop_reg("rdx"))
                },
                3 => {
                    asm_line!(result, "{}", asm_boilerplate::pop_reg("rcx"))
                },
                4 => {
                    asm_line!(result, "{}", asm_boilerplate::pop_reg("r8"))
                },
                5 => {
                    asm_line!(result, "{}", asm_boilerplate::pop_reg("r9"))
                },
                6.. => {//stack, pushed r -> l, so that the args are popped l -> r
                    //note that the for loop is reversed, so that the stack is pushed r->l
                    //data is already on the stack, so leave it
                }
            }
        }



        asm_line!(result, "call {}", self.func_name);
        asm_line!(result, "{}", asm_boilerplate::push_reg(&return_reg_name));//put return value on stack

        result
        
    }
}