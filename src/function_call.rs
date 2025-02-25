use crate::{asm_boilerplate, asm_generation::{self, asm_comment, asm_line, LogicalRegister, PhysicalRegister, RegisterName}, compilation_state::{functions::FunctionList, stack_variables::StackVariables}, expression::Expression, function_declaration::FunctionDeclaration, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size::MemoryLayout, type_info::DataType};
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
            let func_decl = accessible_funcs.get_function(&func_name).expect("found function call but no corresponding function definition");//for recursive functions this is fine, right?
            let num_args_on_stack = if args.len() <= 6 {0} else {args.len() - 6};//first 6 args in registers
            let stack_used_by_args = MemoryLayout::from_bytes(8*num_args_on_stack);//stack args on stack in 8 byte chunks
            let stack_height_bytes = local_variables.get_stack_used() + stack_used_by_args;
            let wanted_extra_stack = MemoryLayout::from_bytes(
                (16 - stack_height_bytes.size_bytes() % 16) % 16//finds the number of extra bytes needed to round to a 16 byte boundary
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

        asm_comment!(result, "calling function: {}", self.func_name);

        asm_line!(result, "sub rsp, {} ;align the stack", self.extra_stack_for_alignment.size_bytes());

        for (i, arg) in self.args.iter().enumerate().rev() {//go through each arg and param right to left
            let param_type = &self.decl.params[i.min(self.decl.params.len()-1)];//when len(params) > len(args), grab the last of params, as it could be a varadic param
            assert!(i < self.decl.params.len());//varadic params not supported yet

            asm_line!(result, "{}", arg.generate_assembly());//calculate the arg
            asm_line!(result, "{}", asm_boilerplate::cast_from_stack(&arg.get_data_type(), param_type.get_type()));//cast to requested type

            asm_line!(result, "{}", asm_boilerplate::pop_reg(&param_type.get_type().memory_size(), &LogicalRegister::ACC));//pop to accumulator temporarily
            
            asm_line!(result, "{}", asm_boilerplate::push_reg(&MemoryLayout::from_bytes(8), &LogicalRegister::ACC));//extend to 8 bytes, without conversion/casting

            if i >= 6 {
                continue;//arg already on stack, and no more registers available
            }

            //else, variable has to go in a 64 bit register
            asm_line!(result, "{}", asm_boilerplate::pop_reg(&MemoryLayout::from_bytes(8), &asm_generation::generate_param_reg(i)));//store the param in the correct register
        }

        asm_line!(result, "call {}", self.func_name);

        asm_line!(result, "add rsp, {} ;remove alignment gap from the stack", self.extra_stack_for_alignment.size_bytes());

        asm_line!(result, "{}", asm_boilerplate::push_reg(&self.decl.return_type.memory_size(), &LogicalRegister::ACC));//put return value on stack

        result
        
    }
}