use memory_size::MemoryLayout;

use crate::{asm_boilerplate, asm_gen_data::AsmData, asm_generation::{self, asm_comment, asm_line, LogicalRegister, RegisterName}, ast_metadata::ASTMetadata, compilation_state::{functions::FunctionList, label_generator::LabelGenerator}, data_type::recursive_data_type::RecursiveDataType, function_declaration::{consume_decl_only, FunctionDeclaration}, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size, parse_data::ParseData, statement::Statement};
use std::fmt::Write;

/**
 * This is a definition of a function
 */
pub struct FunctionDefinition {
    code: Statement,//statement could be a scope if it wants. should this just be a Scope????
    decl: FunctionDeclaration,
    local_scope_data: ParseData//metadata to help with assembly generation
}

impl FunctionDefinition {
    pub fn get_name(&self) -> &str {
        &self.decl.function_name
    }
    pub fn get_return_type(&self) -> RecursiveDataType {
        self.decl.return_type.clone()
    }
    pub fn as_decl(&self) -> FunctionDeclaration {
        self.decl.clone()
    }
    /**
     * consumes tokens to try and make a function definition
     * returns some(function found, remaining tokens) if found, else None
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, global_scope_data: &ParseData) -> Option<ASTMetadata<FunctionDefinition>> {
        //TODO if this function was already declared, you can steal enum variants from it?

        let mut scope_data = global_scope_data.clone_for_new_scope();//clone for a local scope, so that I can have my own declaration in here, and scrap it if things go south

        let ASTMetadata { remaining_slice: after_decl_slice, resultant_tree: func_decl, .. } = consume_decl_only(tokens_queue, previous_queue_idx, &mut scope_data)?;

        if tokens_queue.peek(&after_decl_slice, &scope_data)? == Token::PUNCTUATOR(Punctuator::SEMICOLON) {
            return None;//function declaration + semicolon means no definition for certain
        }
        for i in func_decl.params.iter().rev() {
            scope_data.add_variable(i.get_name(), i.get_type().clone());
        }

        scope_data.add_declaration(func_decl.clone());//so that I can call recursively

        //read the next statement (statement includes a scope)
        //TODO can this _only_ be a scope?
        let ASTMetadata{resultant_tree, remaining_slice} = Statement::try_consume(tokens_queue, &after_decl_slice, accessible_funcs, &mut scope_data)?;
        
        return Some(ASTMetadata{
            resultant_tree: FunctionDefinition {
                code: resultant_tree,
                decl: func_decl,
                local_scope_data: scope_data
            },
            remaining_slice});
    }

    pub fn generate_assembly(&self, label_gen: &mut LabelGenerator, asm_data: &AsmData) -> String {
        //this uses a custom calling convention
        //all params passed on the stack, right to left (caller cleans these up)
        //return value in RAX
        let mut result = String::new();

        //clone myself, but add all my local variables, and add my return type
        let asm_data = &asm_data.clone_for_new_scope(&self.local_scope_data, self.get_return_type());

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
                asm_line!(result, "{}", asm_boilerplate::push_reg(&param.get_type().memory_size(asm_data), &LogicalRegister::ACC));//push how many bits I actually need
                param_stack_used += param.get_type().memory_size(asm_data);
            } else {
                let param_reg = asm_generation::generate_param_reg(param_idx);
                asm_line!(result, "{}", asm_boilerplate::push_reg(&param.get_type().memory_size(asm_data), &param_reg));//truncate param reg to desired size, then push to stack
                param_stack_used += param.get_type().memory_size(asm_data);
            }

        }

        let stack_used_for_body = self.code.get_stack_height(asm_data).unwrap();

        let total_stack_used = stack_used_for_body + param_stack_used;

        let stack_needed_until_aligned =  MemoryLayout::from_bytes(
            (16 - total_stack_used.size_bytes() % 16) % 16//finds the number of extra bytes needed to round to a 16 byte boundary
        );

        let stack_add = stack_used_for_body + stack_needed_until_aligned;
        asm_line!(result, "sub rsp, {} ;allocate stack for local variables and alignment", stack_add.size_bytes());

        asm_line!(result, "{}", self.code.generate_assembly(label_gen, asm_data));

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