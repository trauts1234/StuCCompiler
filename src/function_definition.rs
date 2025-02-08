use memory_size::MemoryLayout;

use crate::{asm_boilerplate, ast_metadata::ASTMetadata, label_generator::LabelGenerator, lexer::{token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size, stack_variables::StackVariables, statement::Statement, type_info::TypeInfo};
use std::fmt::Write;

/**
 * This is a definition of a function
 */
#[derive(Debug)]
pub struct FunctionDefinition {
    return_type: Vec<TypeInfo>,
    function_name: String,
    code: Statement,//statement could be a scope if it wants
    //params: Declaration,
    stack_required: MemoryLayout
    
}

impl FunctionDefinition {
    /**
     * consumes tokens to try and make a function definition
     * returns some(function found, remaining tokens) if found, else None
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice) -> Option<ASTMetadata<FunctionDefinition>> {
        let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        let mut return_data = Vec::new();

        //try and consume as many type specifiers as possible
        loop {
            if let Token::TYPESPECIFIER(ts) = tokens_queue.peek(&curr_queue_idx)? {
                return_data.push(ts.clone());
                tokens_queue.consume(&mut curr_queue_idx);
            } else {
                break;
            }
        }

        if return_data.len() == 0 {
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
        if Token::PUNCTUATION("(".to_owned()) != tokens_queue.consume(&mut curr_queue_idx)? {
            return None;
        }

        //skip over params for now (TODO function params)
        loop {
            if Token::PUNCTUATION(")".to_owned()) == tokens_queue.consume(&mut curr_queue_idx)? {
                break;
            }
        }

        //read the next statement (statement includes a scope)
        let ASTMetadata{resultant_tree, remaining_slice, extra_stack_used} = Statement::try_consume(tokens_queue, &curr_queue_idx, &mut StackVariables::new())?;
        
        return Some(ASTMetadata{
            resultant_tree: FunctionDefinition {
                return_type:return_data,
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
        writeln!(result, "func_{}:", self.function_name).unwrap();
        //create stack frame
        writeln!(result, "push rbp").unwrap();
        writeln!(result, "mov rbp, rsp").unwrap();
        writeln!(result, "sub rsp, {}", self.stack_required.size_bytes()).unwrap();

        //TODO generate stack information, and pass it to the code
        write!(result, "{}", self.code.generate_assembly(label_gen)).unwrap();

        writeln!(result, "{}", asm_boilerplate::func_exit_boilerplate()).unwrap();

        return result;
    }
}