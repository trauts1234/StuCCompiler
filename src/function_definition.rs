use memory_size::MemoryLayout;

use crate::{asm_boilerplate, asm_generation::{self, asm_comment, asm_line, RegisterName}, ast_metadata::ASTMetadata, compilation_state::{functions::FunctionList, label_generator::LabelGenerator, stack_variables::StackVariables}, declaration::{try_consume_declaration_modifiers, Declaration}, function_declaration::FunctionDeclaration, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size, statement::Statement, type_info::{DataType, DeclModifier}};
use std::fmt::Write;

/**
 * This is a definition of a function
 */
#[derive(Debug)]
pub struct FunctionDefinition {
    code: Statement,//statement could be a scope if it wants
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

        //find the brackets enclosing the params
        let args_location = TokenQueueSlice { 
            index: curr_queue_idx.index + 1,//+1 to avoid storing the open bracket in the sub-expression 
            max_index: tokens_queue.find_matching_close_bracket(curr_queue_idx.index) 
        };

        //pop the ( at the start of the params
        if Token::PUNCTUATOR(Punctuator::OPENCURLY) != tokens_queue.consume(&mut curr_queue_idx)? {
            return None;
        }

        let args_segments = tokens_queue.split_outside_parentheses(&args_location, |x| *x == Token::PUNCTUATOR(Punctuator::COMMA));

        //grab all the args
        let mut args = Vec::new();
        if args_location.get_slice_size() >= 1{//ensure there is text between the brackets
            for arg_segment in args_segments {
                args.push(consume_fn_arg(tokens_queue, &arg_segment)?);
            }
        }

        curr_queue_idx.index = args_location.max_index;//jump to end of args

        //pop the ) at the end of the params
        if Token::PUNCTUATOR(Punctuator::CLOSECURLY) != tokens_queue.consume(&mut curr_queue_idx)? {
            return None;
        }

        let return_type = DataType {
            type_info: return_type,
            modifiers: return_modifiers
        };

        //put args on stack variables backwards as args are pushed r->l
        let mut func_body_stack = StackVariables::new_in_func_body(args.iter().rev().cloned().collect(), &return_type);//create a stack and tell it the params and return type of the function

        //read the next statement (statement includes a scope)
        let ASTMetadata{resultant_tree, remaining_slice, extra_stack_used} = Statement::try_consume(tokens_queue, &curr_queue_idx, &mut func_body_stack, accessible_funcs)?;
        
        return Some(ASTMetadata{
            resultant_tree: FunctionDefinition {
                code: resultant_tree,
                stack_required: extra_stack_used,
                decl: FunctionDeclaration {
                    function_name: func_name,
                    params: args,
                    return_type
                }
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
        asm_line!(result, "push rbp");
        asm_line!(result, "mov rbp, rsp");
        asm_line!(result, "sub rsp, {}", self.stack_required.size_bytes());

        asm_comment!(result, "popping args");
        for param_idx in (0..self.decl.params.len()).rev() {
            let param = &self.decl.params[param_idx];
            //args on stack are pushed r->l, so work backwards pushing the register values to the stack
            //calculate smaller register size as data is not 64 bits
            
            if param_idx >= 6 {
                let below_bp_offset = MemoryLayout::from_bytes(16);//8 bytes for return addr, 8 bytes for old bp
                todo!("get offset, put on my stack");//remember 64 bit numbers are args, but I don't want it as 64 bit
            }

            let param_reg = asm_generation::generate_param_reg(param_idx);
            asm_line!(result, "{}", asm_boilerplate::push_reg(&param.data_type.memory_size(), &param_reg));//truncate param reg to desired size, then push to stack
        }

        asm_line!(result, "{}", self.code.generate_assembly(label_gen));

        //destroy stack frame and return
        asm_line!(result, "mov rsp, rbp");
        asm_line!(result, "pop rbp");
        asm_line!(result, "ret");

        return result;
    }
}

fn consume_fn_arg(tokens_queue: &mut TokenQueue, arg_segment: &TokenQueueSlice) -> Option<Declaration> {
    let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(arg_segment);

    let mut data_type_info = Vec::new();

    //try and consume as many type specifiers as possible
    loop {
        if let Token::TYPESPECIFIER(ts) = tokens_queue.peek(&curr_queue_idx)? {
            data_type_info.push(ts.clone());
            tokens_queue.consume(&mut curr_queue_idx);
        } else {
            break;
        }
    }

    if data_type_info.len() == 0 {
        return None;//missing type info
    }

    //by parsing the *x[2] part of int *x[2];, I can get the modifiers and the variable name
    let ASTMetadata{
        resultant_tree: Declaration { data_type: modifiers, name: var_name },
        remaining_slice:_,
        extra_stack_used:_
    } = try_consume_declaration_modifiers(tokens_queue, &curr_queue_idx, &data_type_info)?;

    Some(Declaration {
        data_type: DataType { type_info: data_type_info, modifiers: modifiers.modifiers },
        name: var_name
    })
}