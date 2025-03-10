use memory_size::MemoryLayout;

use crate::{asm_boilerplate, asm_generation::{self, asm_comment, asm_line, LogicalRegister, RegisterName}, ast_metadata::ASTMetadata, compilation_state::{functions::FunctionList, label_generator::LabelGenerator, stack_variables::StackVariables}, data_type::data_type::DataType, function_declaration::{consume_decl_only, FunctionDeclaration}, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size, statement::Statement};
use std::fmt::Write;

/**
 * This is a definition of a function
 */
pub struct FunctionDefinition {
    code: Statement,//statement could be a scope if it wants. should this just be a Scope????
    stack_required: MemoryLayout,
    decl: FunctionDeclaration
}

impl FunctionDefinition {
    pub fn get_name(&self) -> &str {
        &self.decl.function_name
    }
    pub fn get_return_type(&self) -> DataType {
        self.decl.return_type.clone()
    }
    pub fn as_decl(&self) -> FunctionDeclaration {
        self.decl.clone()
    }
    /**
     * consumes tokens to try and make a function definition
     * returns some(function found, remaining tokens) if found, else None
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList) -> Option<ASTMetadata<FunctionDefinition>> {
        let ASTMetadata { remaining_slice: after_decl_slice, resultant_tree: func_decl, .. } = consume_decl_only(tokens_queue, previous_queue_idx)?;

        if tokens_queue.peek(&after_decl_slice)? == Token::PUNCTUATOR(Punctuator::SEMICOLON) {
            return None;//function declaration + semicolon means no definition for certain
        }

        //put args on stack variables backwards as args are pushed r->l
        //create a stack and tell it the params and return type of the function
        let mut func_body_stack = StackVariables::new_in_func_body(func_decl.params.iter().rev().cloned().collect(), &func_decl.return_type);

        //read the next statement (statement includes a scope)
        let ASTMetadata{resultant_tree, remaining_slice, extra_stack_used} = Statement::try_consume(tokens_queue, &after_decl_slice, &mut func_body_stack, accessible_funcs)?;
        
        return Some(ASTMetadata{
            resultant_tree: FunctionDefinition {
                code: resultant_tree,
                stack_required: extra_stack_used,
                decl: func_decl
            },
            extra_stack_used,
            remaining_slice});
    }

    pub fn generate_assembly(&self, label_gen: &mut LabelGenerator) -> String {
        //this uses a custom calling convention
        //all params passed on the stack, right to left (caller cleans these up)
        //return value in RAX
        let mut result = String::new();

        //set label as same as function name
        asm_line!(result, "{}:", self.decl.function_name);
        //create stack frame
        asm_line!(result, "push rbp ;create stack frame");
        asm_line!(result, "mov rbp, rsp ;''");

        asm_comment!(result, "popping args");
        let mut param_stack_used = MemoryLayout::new();

        for param_idx in (0..self.decl.params.len()).rev() {
            let param = &self.decl.params[param_idx];
            //args on stack are pushed r->l, so work backwards pushing the register values to the stack
            //calculate smaller register size as data is not 64 bits
            
            if param_idx >= 6 {
                let below_bp_offset = MemoryLayout::from_bytes(8);//8 bytes for return addr, as rbp points to the start of the stack frame
                let arg_offset = MemoryLayout::from_bytes(8 + (param_idx - 6) * 8);//first 6 are in registers, each is 8 bytes, +8 as first arg is still +8 extra from bp
                let arg_bp_offset = below_bp_offset + arg_offset;//how much to *add* to bp to go below the stack frame and get the param 

                asm_line!(result, "mov {}, [rbp+{}]", LogicalRegister::ACC.generate_reg_name(&MemoryLayout::from_bytes(8)), arg_bp_offset.size_bytes());//grab as 64 bit
                asm_line!(result, "{}", asm_boilerplate::push_reg(&param.get_type().memory_size(), &LogicalRegister::ACC));//push how many bits I actually need
                param_stack_used += param.get_type().memory_size();
            } else {
                let param_reg = asm_generation::generate_param_reg(param_idx);
                asm_line!(result, "{}", asm_boilerplate::push_reg(&param.get_type().memory_size(), &param_reg));//truncate param reg to desired size, then push to stack
                param_stack_used += param.get_type().memory_size();
            }

        }

        let total_stack_used = self.stack_required + param_stack_used;

        let stack_needed_until_aligned =  MemoryLayout::from_bytes(
            (16 - total_stack_used.size_bytes() % 16) % 16//finds the number of extra bytes needed to round to a 16 byte boundary
        );

        let stack_add = self.stack_required + stack_needed_until_aligned;
        asm_line!(result, "sub rsp, {} ;allocate stack for local variables and alignment", stack_add.size_bytes());

        asm_line!(result, "{}", self.code.generate_assembly(label_gen));

        //destroy stack frame and return

        if self.get_name() == "main" {
            //main auto returns 0
            asm_line!(result, "mov rax, 0");
        }
        asm_line!(result, "mov rsp, rbp");
        asm_line!(result, "pop rbp");
        asm_line!(result, "ret");

        return result;
    }
}