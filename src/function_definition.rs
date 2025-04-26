use memory_size::MemorySize;
use crate::{asm_gen_data::{AsmData, GlobalAsmData}, assembly::{assembly::Assembly, operand::{generate_param_reg, immediate::{ImmediateValue, MemorySizeExt}, memory_operand::MemoryOperand, register::Register, Operand, RegOrMem}, operation::AsmOperation}, ast_metadata::ASTMetadata, compilation_state::{functions::FunctionList, label_generator::LabelGenerator}, compound_statement::ScopeStatements, data_type::{base_type::BaseType, recursive_data_type::DataType}, debugging::{ASTDisplay, DebugDisplay}, function_call::aligned_size, function_declaration::{consume_decl_only, FunctionDeclaration}, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, parse_data::ParseData};
use unwrap_let::unwrap_let;

/**
 * This is a definition of a function
 */
pub struct FunctionDefinition {
    code: ScopeStatements,//statement could be a scope if it wants. should this just be a Scope????
    decl: FunctionDeclaration,
    local_scope_data: ParseData//metadata to help with assembly generation
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
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, global_scope_data: &ParseData, struct_label_gen: &mut LabelGenerator) -> Option<ASTMetadata<FunctionDefinition>> {
        //TODO if this function was already declared, you can steal enum variants from it?

        let mut scope_data = global_scope_data.clone_for_new_scope();//clone for a local scope, so that I can have my own declaration in here, and scrap it if things go south

        let ASTMetadata { remaining_slice: after_decl_slice, resultant_tree: func_decl, .. } = consume_decl_only(tokens_queue, previous_queue_idx, &mut scope_data, struct_label_gen)?;

        if tokens_queue.peek(&after_decl_slice, &scope_data)? == Token::PUNCTUATOR(Punctuator::SEMICOLON) {
            return None;//function declaration + semicolon means no definition for certain
        }
        for i in func_decl.params.iter().rev() {
            scope_data.add_variable(i.get_name(), i.get_type().clone());
        }

        scope_data.add_declaration(func_decl.clone());//so that I can call recursively

        let ASTMetadata{resultant_tree, remaining_slice} = ScopeStatements::try_consume(tokens_queue, &after_decl_slice, accessible_funcs, &mut scope_data, struct_label_gen)?;
        
        return Some(ASTMetadata{
            resultant_tree: FunctionDefinition {
                code: resultant_tree,
                decl: func_decl,
                local_scope_data: scope_data
            },
            remaining_slice});
    }

    pub fn generate_assembly(&self, global_asm_data: &mut GlobalAsmData) -> Assembly {
        let mut result = Assembly::make_empty();
        let mut stack_data = MemorySize::new();//stack starts as empty in a function

        //clone myself, but add all my local variables, and add my return type
        let asm_data = &AsmData::for_new_function(&global_asm_data, &self.local_scope_data, self.get_return_type(), &mut stack_data);

        //set label as same as function name
        result.add_instruction(AsmOperation::Label { name: self.decl.function_name.clone() });
        //create stack frame
        result.add_commented_instruction(AsmOperation::CreateStackFrame, "create stack frame");

        let code_for_body = self.code.generate_assembly(asm_data, &mut stack_data, global_asm_data);//calculate stack needed for function, while generating asm
        let aligned_stack_usage = aligned_size(stack_data, MemorySize::from_bytes(16));
        result.add_commented_instruction(AsmOperation::SUB {
            destination: RegOrMem::Reg(Register::_SP),
            decrement: Operand::Imm(aligned_stack_usage.as_imm()),
            data_type: DataType::RAW(BaseType::U64),
        }, "allocate stack for local variables and alignment");

        result.add_comment("moving args to memory");

        //args on stack are pushed r->l, so work backwards pushing the register values to the stack
        for param_idx in (0..self.decl.params.len()).rev() {
            let param = &self.decl.params[param_idx];//get metadata about param
            let param_size = param.get_type().memory_size(asm_data);//get size of param 
            unwrap_let!(Operand::Mem(MemoryOperand::SubFromBP(param_offset)) = &asm_data.get_variable(param.get_name()).location);//get the location of where the param should *end up* since it gets moved from registers to memory
            
            if param_idx >= 6 {
                let below_bp_offset = MemorySize::from_bytes(8);//8 bytes for return addr, as rbp points to the start of the stack frame
                let arg_offset = MemorySize::from_bytes(8 + (param_idx as u64 - 6) * 8);//first 6 are in registers, each is 8 bytes, +8 as first arg is still +8 extra from bp
                let arg_bp_offset = below_bp_offset + arg_offset;//how much to *add* to bp to go below the stack frame and get the param 

                let arg_address_operand = Operand::Mem(MemoryOperand::PreviousStackFrame { add_to_rbp: arg_bp_offset });

                result.add_instruction(AsmOperation::MOV {
                    to: RegOrMem::Reg(Register::acc()),
                    from: arg_address_operand,
                    size: param_size
                });//grab data

                result.add_instruction(AsmOperation::MOV {
                    to: RegOrMem::Mem(MemoryOperand::SubFromBP(*param_offset)),
                    from: Operand::Reg(Register::acc()),
                    size: param_size
                });//store in allocated space
            } else {
                let param_reg = generate_param_reg(param_idx as u64);
                //truncate param reg to desired size
                //then write to its allocated address on the stack
                result.add_instruction(AsmOperation::MOV {
                    to: RegOrMem::Mem(MemoryOperand::SubFromBP(*param_offset)),
                    from: Operand::Reg(param_reg),
                    size: param_size
                });
            }

        }

        result.merge(&code_for_body);

        //destroy stack frame and return

        if self.get_name() == "main" {
            //main auto returns 0
            result.add_instruction(AsmOperation::MOV {
                to: RegOrMem::Reg(Register::acc()),
                from: Operand::Imm(ImmediateValue("0".to_string())),
                size: MemorySize::from_bytes(8)
            });
        }
        
        //destroy stack frame and return
        result.add_instruction(AsmOperation::DestroyStackFrame);
        result.add_instruction(AsmOperation::Return);

        return result;
    }
}

impl ASTDisplay for FunctionDefinition {
    fn display_ast(&self, f: &mut crate::debugging::TreeDisplayInfo) {
        f.write(&self.decl.display());
        f.indent();
        self.code.display_ast(f);
        f.dedent();
    }
}