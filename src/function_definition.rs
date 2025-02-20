use memory_size::MemoryLayout;

use crate::{asm_generation::asm_line, ast_metadata::ASTMetadata, compilation_state::{functions::FunctionList, label_generator::LabelGenerator, stack_variables::StackVariables}, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size, statement::Statement, type_info::{DataType, DeclModifier}};
use std::fmt::Write;

/**
 * This is a definition of a function
 */
#[derive(Debug)]
pub struct FunctionDefinition {
    return_type: DataType,
    function_name: String,
    code: Statement,//statement could be a scope if it wants
    //params: Declaration,
    stack_required: MemoryLayout
    
}

impl FunctionDefinition {
    pub fn get_name(&self) -> &str {
        &self.function_name
    }
    pub fn get_return_type(&self) -> DataType {
        self.return_type.clone()
    }
    /**
     * consumes tokens to try and make a function definition
     * returns some(function found, remaining tokens) if found, else None
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList) -> Option<ASTMetadata<FunctionDefinition>> {
        let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        let mut return_type = Vec::new();
        let mut return_modifiers = Vec::new();

        //try and consume as many type specifiers as possible
        while let Token::TYPESPECIFIER(ts) = tokens_queue.peek(&curr_queue_idx)? {
            return_type.push(ts.clone());
            tokens_queue.consume(&mut curr_queue_idx);
        }

        while Token::PUNCTUATOR(Punctuator::ASTERISK) == tokens_queue.peek(&curr_queue_idx)? {
            return_modifiers.push(DeclModifier::POINTER);
            tokens_queue.consume(&mut curr_queue_idx);
        }

        if return_type.len() == 0 {
            return None;//missing type info
        }

        //try to match an identifier, to find out the function name

        let func_name = 
        if let Token::IDENTIFIER(ident) = tokens_queue.consume(&mut curr_queue_idx)? {
            ident.to_string()
        }
        else {
            return None;
        };

        //pop the ( after the function name
        if Token::PUNCTUATOR(Punctuator::OPENCURLY) != tokens_queue.consume(&mut curr_queue_idx)? {
            return None;
        }

        //skip over params for now (TODO function params)
        loop {
            if Token::PUNCTUATOR(Punctuator::CLOSECURLY) == tokens_queue.consume(&mut curr_queue_idx)? {
                break;
            }
        }

        let return_type = DataType {
            type_info: return_type,
            modifiers: return_modifiers
        };

        let mut func_body_stack = StackVariables::new_in_func_body(&return_type);//create a stack and tell it the return type of the function

        //read the next statement (statement includes a scope)
        let ASTMetadata{resultant_tree, remaining_slice, extra_stack_used} = Statement::try_consume(tokens_queue, &curr_queue_idx, &mut func_body_stack, accessible_funcs)?;
        
        return Some(ASTMetadata{
            resultant_tree: FunctionDefinition {
                return_type,
                function_name: func_name,
                code: resultant_tree,
                stack_required: extra_stack_used
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
        asm_line!(result, "{}:", self.function_name);
        //create stack frame
        asm_line!(result, "push rbp");
        asm_line!(result, "mov rbp, rsp");
        asm_line!(result, "sub rsp, {}", self.stack_required.size_bytes());

        asm_line!(result, "{}", self.code.generate_assembly(label_gen));

        //destroy stack frame and return
        asm_line!(result, "mov rsp, rbp");
        asm_line!(result, "pop rbp");
        asm_line!(result, "ret");

        return result;
    }
}