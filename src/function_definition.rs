use stack_management::simple_stack_frame::SimpleStackFrame;
use crate::{args_handling::location_allocation::{generate_param_and_return_locations, AllocatedLocation, EightByteLocation, ReturnLocation}, asm_gen_data::{AsmData, GlobalAsmData}, assembly::{assembly::IRCode, operand::{ memory_operand::MemoryOperand, register::GPRegister, Storage, STACK_ALIGN}, operation::{IROperation, CalleeReturnData, Label, ReadParamFromMem, ReadParamFromReg}}, ast_metadata::ASTMetadata, compound_statement::ScopeStatements, data_type::{base_type::{BaseType, IntegerType}, recursive_data_type::DataType}, debugging::ASTDisplay, function_declaration::{consume_decl_only, FunctionDeclaration}, generate_ir_traits::GenerateIR, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, number_literal::typed_value::NumberLiteral, parse_data::ParseData};
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
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, global_scope_data: &ParseData) -> Option<ASTMetadata<FunctionDefinition>> {
        //TODO if this function was already declared, you can steal enum variants from it?

        let mut scope_data = global_scope_data.clone_for_new_scope();//clone for a local scope, so that I can have my own declaration in here, and scrap it if things go south

        let ASTMetadata { remaining_slice: after_decl_slice, resultant_tree: func_decl, .. } = consume_decl_only(tokens_queue, previous_queue_idx, &mut scope_data)?;

        if tokens_queue.peek(&after_decl_slice, &scope_data)? == Token::PUNCTUATOR(Punctuator::SEMICOLON) {
            return None;//function declaration + semicolon means no definition for certain
        }
        for i in func_decl.params.iter().rev() {
            scope_data.add_variable(&i.name, i.data_type.clone());
        }

        scope_data.add_declaration(func_decl.clone());//so that I can call recursively

        let ASTMetadata{resultant_tree, remaining_slice} = ScopeStatements::try_consume(tokens_queue, &after_decl_slice, &mut scope_data)?;
        
        return Some(ASTMetadata{
            resultant_tree: FunctionDefinition {
                code: resultant_tree,
                decl: func_decl,
                local_scope_data: scope_data
            },
            remaining_slice});
    }

    //cannot be GenerateIR as it creates some things like `stack_data`
    pub fn generate_assembly(&self, global_asm_data: &mut GlobalAsmData) -> (IRCode, SimpleStackFrame) {
        let mut result = IRCode::make_empty();
        //as per SYSV ABI, stack is aligned (once stack frame generated) to 16 bytes
        let mut stack_data = SimpleStackFrame::new(STACK_ALIGN);//stack starts as empty in a function
        let (return_location, args_locations) = generate_param_and_return_locations(self.decl.params.iter().map(|decl| &decl.data_type), &self.get_return_type(), global_asm_data);

        //clone myself, but add all my local variables, and add my return type
        let asm_data = &AsmData::for_new_function(&global_asm_data, &self.local_scope_data, self.get_return_type(), return_location, &mut stack_data);

        //set label as same as function name
        result.add_instruction(IROperation::Label(Label::Global(self.decl.function_name.clone())));
        //create stack frame
        result.add_commented_instruction(IROperation::CreateStackFrame, "create stack frame");

        let (code_for_body, _) = self.code.generate_ir(asm_data, &mut stack_data, global_asm_data);//calculate stack needed for function, while generating asm

        result.add_comment("moving args to memory");

        //varadic args not implemented yet
        assert!(self.decl.params.last().is_none_or(|x| x.data_type != DataType::RAW(BaseType::VaArg)));

        //calculate where each arg is, and split into lists
        let mut reg_args = Vec::new();
        let mut mem_args = Vec::new();

        // let mut memory_offset_tracker = MemorySize::new();
        for param_idx in 0..self.decl.params.len() {
            let param = &self.decl.params[param_idx];//get metadata about param
            let param_size = param.data_type.memory_size(asm_data);//get size of param

            let param_start_location = args_locations[param_idx].clone();
            unwrap_let!(MemoryOperand::SubFromBP(param_destination) = asm_data.get_variable(&param.name).location);//get the location of where the param should *end up* since it gets moved to a new location

            match param_start_location {
                AllocatedLocation::Regs(eightbyte_locations) => 
                    reg_args.push(ReadParamFromReg{ eightbyte_locations, param_size, param_destination }),
                AllocatedLocation::Memory => 
                    mem_args.push(ReadParamFromMem{ param_size, param_destination }),
            }
        }

        result.add_instruction(IROperation::ReadParams { regs: reg_args, mem: mem_args });

        // //go through register args first, as they are very likely to be clobbered if I wait too long...
        // for (eight_byte_locations, param_size, param_end_location, param_idx) in reg_args {
        //     let mut how_far_into_param = MemorySize::new();//when reading multiple regs, I need the results in sequential eightbytes
        //     for (i,location) in eight_byte_locations.iter().enumerate() {
        //         //if I am reading eightbytes in the middle of a struct, each eightbyte is obviously 8 bytes
        //         //but the last eightbyte could just be a few bits remaining on a struct
        //         let eightbyte_size = MemorySize::from_bytes(param_size.size_bytes() - 8*(i as u64))//remainder size = size of struct - eightbytes consumed
        //             .min(MemorySize::from_bytes(8));//but only up to eight bytes, to fit in a register

        //         let from_reg = match location {
        //             EightByteLocation::GP(gpregister) => Operand::GPReg(*gpregister),
        //             EightByteLocation::XMM(mmregister) => Operand::MMReg(*mmregister),
        //         };

        //         //rax should be safe to clobber here...
        //         result.add_instruction(AsmOperation::LEA { from: MemoryOperand::SubFromBP(*param_end_location)});
        //         result.add_instruction(AsmOperation::ADD { increment: Operand::Imm(how_far_into_param.as_imm()), data_type: ScalarType::Integer(IntegerType::U64) });

        //         result.add_commented_instruction(AsmOperation::MOV {
        //             to: RegOrMem::Mem(MemoryOperand::MemoryAddress { pointer_reg: GPRegister::acc() }),
        //             from: from_reg,
        //             size: eightbyte_size
        //         }, format!("moving reg arg to memory (param no.{} eightbyte no.{})", param_idx, i));

        //         how_far_into_param += eightbyte_size;//write to next part of struct/union
        //     }
        // }
        // // go through memory args last
        // for (param_size, param_end_location, param_idx) in mem_args {
        //     let skip_stackframe_and_return_addr = MemorySize::from_bytes(16);// +8 to skip stack frame, +8 to skip return address, now points to first memory arg

        //     let arg_address_operand = MemoryOperand::PreviousStackFrame { add_to_rbp: skip_stackframe_and_return_addr + memory_offset_tracker };
        //     memory_offset_tracker += param_size.align_up(&MemorySize::from_bytes(8));//args are padded, so keep track of the memory address here

        //     result.add_commented_instruction(AsmOperation::LEA {
        //         from: arg_address_operand,
        //     }, format!("getting pointer to stack arg (param no.{})", param_idx));//grab pointer to data
        //     result.add_instruction(AsmOperation::MOV { to: RegOrMem::GPReg(GPRegister::_SI), from: Operand::GPReg(GPRegister::acc()), size: PTR_SIZE });

        //     result.add_commented_instruction(AsmOperation::LEA {//Hope this doesn't clobber DI
        //         from: MemoryOperand::SubFromBP(*param_end_location),
        //     }, format!("getting pointer to destination (param no.{})", param_idx));//grab pointer to resultant location
        //     result.add_instruction(AsmOperation::MOV { to: RegOrMem::GPReg(GPRegister::_DI), from: Operand::GPReg(GPRegister::acc()), size: PTR_SIZE });

        //     result.add_instruction(AsmOperation::MEMCPY { size: param_size });//copy the param
        // }

        result.merge(&code_for_body);
        
        //destroy stack frame and return
        result.add_instruction(IROperation::DestroyStackFrame);
        if self.get_name() == "main" {
            //main automatically returns 0
            result.add_instruction(IROperation::Return{
                return_data: Some((
                    CalleeReturnData::InRegs(vec![EightByteLocation::GP(GPRegister::acc())]),
                    Storage::Constant(NumberLiteral::INTEGER { data: 0, data_type: IntegerType::I32 })
                ))
            });
        } else {
            result.add_instruction(IROperation::Return{ return_data: None });
        }

        return (result, stack_data);
    }
}

impl ASTDisplay for FunctionDefinition {
    fn display_ast(&self, f: &mut crate::debugging::TreeDisplayInfo) {
        f.write(&format!("{}", self.decl));
        f.indent();
        self.code.display_ast(f);
        f.dedent();
    }
}