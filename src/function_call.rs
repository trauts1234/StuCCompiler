use crate::{args_handling::location_allocation::{generate_param_and_return_locations, AllocatedLocation, EightByteLocation}, asm_boilerplate::cast_from_acc, asm_gen_data::AsmData, assembly::{assembly::Assembly, operand::{immediate::ImmediateValue, memory_operand::MemoryOperand, register::GPRegister, Operand, RegOrMem}, operation::AsmOperation}, compilation_state::label_generator::LabelGenerator, data_type::{base_type::{BaseType, FloatType, ScalarType}, recursive_data_type::{calculate_unary_type_arithmetic, DataType}}, debugging::ASTDisplay, expression::expression::{self, Expression}, expression_visitors::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor, put_scalar_in_acc::ScalarInAccVisitor, put_struct_on_stack::CopyStructVisitor}, function_declaration::FunctionDeclaration, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, parse_data::ParseData, stack_allocation::{align, StackAllocator}};
use memory_size::MemorySize;
use unwrap_let::unwrap_let;

#[derive(Clone, Debug)]
pub struct FunctionCall {
    func_name: String,//maybe an enum, for function pointers
    args: Vec<Expression>,

    decl: FunctionDeclaration
}

impl FunctionCall {
    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_func_call(self)
    }
    
    pub fn generate_assembly_scalar_return(&self, visitor: &mut ScalarInAccVisitor) -> Assembly {
        //system V ABI
        let mut result = Assembly::make_empty();

        result.add_comment(format!("calling function: {}", self.func_name));

        //attach type to each of the args
        let type_matched_args: Vec<_> = self.args.iter()
            .enumerate()
            .map(|(i, expr)|{
                let param_type = if i >= self.decl.params.len() {
                    assert!(self.decl.params.last().unwrap().data_type == DataType::new(BaseType::VaArg));//more args than params, so must be varadic
                    //promotion of the arg is required, subject to some funny rules
                    match expr.accept(&mut GetDataTypeVisitor {asm_data: visitor.asm_data}).decay() {
                        DataType::RAW(BaseType::Scalar(ScalarType::Float(_))) => DataType::RAW(BaseType::Scalar(ScalarType::Float(FloatType::F64))),//for some reason, varadic args request promotion to f64
                        x => calculate_unary_type_arithmetic(&x)//promote the param via C99, §6.5.2.2/6
                    }
                } else {
                    self.decl.params[i].data_type.clone()//arg gets cast to param type
                };

                (param_type, expr)
            })
            .collect();

        let (return_location, params_locations) = generate_param_and_return_locations(type_matched_args.iter().map(|(t, _)| t), &self.decl.return_type, visitor.asm_data);

        assert!(type_matched_args.iter().all(|x| x.0.decay() == x.0));//none can be array at this point

        //maintaining order, split into categories based on location allocated
        let mut memory_args = Vec::new();
        let mut reg_args = Vec::new();
        for ((dtype, expr), location) in type_matched_args.iter().zip(params_locations.iter()) {
            match location {
                AllocatedLocation::Regs(regs) => reg_args.push((dtype, expr, regs)),
                AllocatedLocation::Memory => memory_args.push((dtype, expr)),
            }
        }

        //allocate memory, ensuring memory args are allocated on top of everything else
        let regs_with_spill_space: Vec<_> = reg_args.into_iter()
            .map(|(dtype, expr, reg)| {
                let mem_required = dtype.memory_size(visitor.asm_data);
                assert!(mem_required.size_bytes() / reg.len() as u64 <= 8);//ensure there are only 8 bytes or less per register being used

                (dtype, expr, reg, MemoryOperand::SubFromBP(visitor.stack_data.allocate(mem_required)))
            })
            .collect();

        // this is used to alloc offsets from RSP, so as long as this is made such that SP is aligned, the rest should be?
        let mut stack_used_by_mem_args = MemorySize::default();
        let memory_args_allocated: Vec<_> = memory_args.into_iter()
            .rev()//apply to the args on the top of the stack first
            .map(|(dtype, expr)| {
                let mem_required = dtype.memory_size(visitor.asm_data);
                let location = MemoryOperand::AddToSP(stack_used_by_mem_args);
                stack_used_by_mem_args += mem_required;//step over the param
                stack_used_by_mem_args += align(stack_used_by_mem_args, MemorySize::from_bytes(8));//align correctly
                (dtype, expr, location)
            })
            .rev()//undo the previous .rev()
            .collect();

        //calculate values and put in spill space
        for (dtype, expr, _, spill_space) in &regs_with_spill_space {
            let calculate = put_arg_on_stack(expr, (*dtype).clone(), spill_space.clone(), visitor.asm_data, visitor.stack_data, visitor.global_asm_data);
            result.merge(&calculate);
        }

        //alloc stack for the memory args
        result.add_commented_instruction(AsmOperation::AllocateStack(stack_used_by_mem_args), "allocating stack for memory args");
        //calculate memory args and put in the correct place
        for (dtype, expr, location) in memory_args_allocated {
            let calculate = put_arg_on_stack(expr, dtype.clone(), location, visitor.asm_data, visitor.stack_data, visitor.global_asm_data);
            result.merge(&calculate);
        }

        //pop data stored on stack into the right registers
        for (_, _, regs, spill_space) in &regs_with_spill_space {

            for (i, reg) in regs.iter().enumerate() {
                //for each eightbyte, read further into the arg
                let eightbyte_offset = MemorySize::from_bytes(8 * (i as u64));
                unwrap_let!(MemoryOperand::SubFromBP(spill_base) = spill_space);
                //spill base takes us towards SP, then offset walks us back towards BP
                let eightbyte_addr = MemoryOperand::SubFromBP(*spill_base - eightbyte_offset);
                // convert to an operand for assembly
                let to_location = match reg {
                    EightByteLocation::GP(gpregister) => RegOrMem::GPReg(*gpregister),
                    EightByteLocation::XMM(mmregister) => RegOrMem::MMReg(*mmregister),
                };

                result.add_commented_instruction(AsmOperation::MOV {
                    to: to_location,
                    from: Operand::Mem(eightbyte_addr),
                    size: MemorySize::from_bytes(8),
                }, format!("moving eightbyte {} into the appropriate register", i));
            }
        }

        let fp_args = regs_with_spill_space.iter()
            .flat_map(|(_, _, x, _)| x.iter())//get each reg recursively
            .filter(|reg| if let EightByteLocation::XMM(_) = reg {true} else {false})//filter xmm regs only
            .count();
        result.add_instruction(AsmOperation::MOV { to: RegOrMem::GPReg(GPRegister::_AX), from: Operand::Imm(ImmediateValue(fp_args.to_string())), size: MemorySize::from_bytes(8) });

        result.add_instruction(AsmOperation::CALL { label: self.func_name.clone() });

        result.add_commented_instruction(AsmOperation::DeallocateStack(stack_used_by_mem_args), "deallocating stack for memory args");

        result
    }

    pub fn get_callee_decl(&self) -> &FunctionDeclaration {
        &self.decl
    }
    
    pub fn try_consume_whole_expr(tokens_queue: &TokenQueue, curr_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData, struct_label_gen: &mut LabelGenerator) -> Option<FunctionCall> {
        //look for unary postfixes as association is left to right
        let last_token = tokens_queue.peek_back(&curr_queue_idx, &scope_data)?;
    
        if last_token != Token::PUNCTUATOR(Punctuator::CLOSECURLY){
            return None;
        }
    
        let curly_open_idx = tokens_queue.find_matching_open_bracket(curr_queue_idx.max_index-1);//-1 as max index is exclusive
    
        let all_args_slice = TokenQueueSlice {
            index: curly_open_idx+1,
            max_index: curr_queue_idx.max_index-1
        };

        let args_slices = tokens_queue.split_outside_parentheses(&all_args_slice, |x| *x == Token::PUNCTUATOR(Punctuator::COMMA), &TokenSearchType::skip_all_brackets());

        let mut args = Vec::new();

        if all_args_slice.get_slice_size() > 0 {//ensure args have actually been passed
            for arg_slice in args_slices {
                args.push(expression::try_consume_whole_expr(tokens_queue, &arg_slice, scope_data, struct_label_gen)?);
            }
        }

        let func_slice = TokenQueueSlice {//array or pointer etc.
            index: curr_queue_idx.index,
            max_index: curly_open_idx
        };

        if let Token::IDENTIFIER(func_name) = tokens_queue.peek(&func_slice, &scope_data)? {
            //warning: label definition in a bad place will trip this up:

            //label: printf(""); will be interpreted at label()
            let func_decl = scope_data.get_function_declaration(&func_name).expect(&format!("found function call but no corresponding function declaration: {}", func_name));
            Some(FunctionCall {
                func_name, 
                args,
                decl: func_decl.clone(),
            })
        } else {
            None
        }
    }
}

impl ASTDisplay for FunctionCall {
    fn display_ast(&self, f: &mut crate::debugging::TreeDisplayInfo) {
        f.write(&format!("call {}", self.func_name));
        f.indent();
        for i in &self.args {
            i.display_ast(f);
        }
        f.dedent();
    }
}

fn put_arg_on_stack(expr: &Expression, arg_type: DataType,location: MemoryOperand, asm_data: &AsmData, stack_data: &mut StackAllocator, global_asm_data:&mut crate::asm_gen_data::GlobalAsmData) -> Assembly {
    let mut result = Assembly::make_empty();
    //push arg to stack
    let param_type = expr.accept(&mut GetDataTypeVisitor{asm_data});

    match (&param_type.decay(), &arg_type) {
        (DataType::RAW(BaseType::Struct(_)), _) => {
            result.add_comment("putting struct arg on stack");
            let struct_clone_asm = expr.accept(&mut CopyStructVisitor{asm_data, stack_data, global_asm_data, resultant_location: Operand::Mem(location) });
            result.merge(&struct_clone_asm);

        },
        (original_type, casted_type) => {
            result.add_comment("putting arg on stack");
            assert!(original_type.memory_size(asm_data).size_bytes() <= 8);

            let arg_expr_asm = expr.accept(&mut ScalarInAccVisitor {asm_data, stack_data, global_asm_data});
            result.merge(&arg_expr_asm);//put value in acc, using standard stack to calculate it

            let arg_cast_asm = cast_from_acc(original_type, casted_type, asm_data);
            result.merge(&arg_cast_asm);

            result.add_instruction(AsmOperation::MOV {
                to: RegOrMem::Mem(location),
                from: Operand::GPReg(GPRegister::acc()),
                size: casted_type.memory_size(asm_data)//do I need to zero the upper bytes or sth?
            });
        }
    }

    result
}