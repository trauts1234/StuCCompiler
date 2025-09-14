
use colored::Colorize;
use stack_management::{simple_stack_frame::SimpleStackFrame, stack_item::StackItemKey};
use unwrap_let::unwrap_let;
use memory_size::MemorySize;
use crate::{asm_gen_data::{AsmData, GlobalAsmData}, assembly::{assembly::Assembly, operand::{immediate::ToImmediate, memory_operand::MemoryOperand, register::GPRegister, Operand, Storage, PTR_SIZE}, operation::AsmOperation}, data_type::{base_type::{BaseType, IntegerType, ScalarType}, recursive_data_type::{calculate_promoted_type_arithmetic, calculate_unary_type_arithmetic, DataType}}, debugging::ASTDisplay, expression::{binary_expression_operator::BinaryExpressionOperator, expression::{generate_assembly_for_assignment, Expression}}, expression_visitors::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor, reference_assembly_visitor::ReferenceVisitor}, generate_ir::GenerateIR, number_literal::typed_value::NumberLiteral};

#[derive(Clone, Debug)]
pub struct BinaryExpression {
    lhs: Box<Expression>,
    operator: BinaryExpressionOperator,
    rhs: Box<Expression>,
}

impl BinaryExpression {
    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_binary_expression(self)
    }
}
impl GenerateIR for BinaryExpression {
    fn generate_ir(&self, asm_data: &AsmData, stack_data: &mut SimpleStackFrame, global_asm_data: &GlobalAsmData) -> (Assembly, Option<StackItemKey>) {
        let mut result = Assembly::make_empty();

        if self.operator == BinaryExpressionOperator::Assign {
            return generate_assembly_for_assignment(&self.lhs, &self.rhs, asm_data, stack_data, global_asm_data);
        }

        let lhs_type = self.lhs.accept(&mut GetDataTypeVisitor {asm_data});
        let rhs_type = self.rhs.accept(&mut GetDataTypeVisitor {asm_data});

        result.add_comment("generating lhs and rhs of binary expression");
        let (lhs_asm, lhs_result) = self.lhs.generate_ir(asm_data, stack_data, global_asm_data);
        result.merge(&lhs_asm);
        let (rhs_asm, rhs_result) = self.rhs.generate_ir(asm_data, stack_data, global_asm_data);
        result.merge(&rhs_asm);

        let promoted_type = match &self.operator {//I already have a function for this?
            BinaryExpressionOperator::Assign => panic!("assignment already done"),
            x if x.as_boolean_instr().is_some() => DataType::RAW(BaseType::Scalar(ScalarType::Integer(IntegerType::_BOOL))),//is a boolean operator, operands are booleans
            _ => calculate_promoted_type_arithmetic(&lhs_type, &rhs_type)//else find a common meeting ground
        };
        let promoted_size = promoted_type.memory_size(asm_data);

        //where the result of the computation goes
        let resultant_location = stack_data.allocate(promoted_size);

        match &self.operator {
            BinaryExpressionOperator::Add => {
                result.add_comment(format!("adding {} numbers", promoted_size));

                let (ptr_scale_asm, lhs_scaled, rhs_scaled) = apply_pointer_scaling(lhs_result, rhs_result, &promoted_type, asm_data, stack_data, global_asm_data);

                result.merge(&ptr_scale_asm);

                result.add_instruction(AsmOperation::ADD {
                    lhs: Storage::Stack(lhs_scaled),
                    rhs: Storage::Stack(rhs_scaled),
                    to: Storage::Stack(resultant_location),
                    data_type: promoted_type.decay_to_primative(),
                });
                
            },
            BinaryExpressionOperator::Subtract => {
                result.add_comment(format!("subtracting {} numbers", promoted_size));

                let (ptr_scale_asm, lhs_scaled, rhs_scaled) = apply_pointer_scaling(&self.lhs, &self.rhs, &promoted_type, asm_data, stack_data, global_asm_data);

                result.merge(&ptr_scale_asm);

                result.add_instruction(AsmOperation::SUB {
                    lhs: Storage::Stack(lhs_scaled),
                    rhs: Storage::Stack(rhs_scaled),
                    to: Storage::Stack(resultant_location),
                    data_type: promoted_type.decay_to_primative(),
                });

            }
            BinaryExpressionOperator::Multiply => {
                result.add_comment("mulitplying numbers");

                result.merge(&put_lhs_ax_rhs_cx(&self.lhs, &promoted_type, &self.rhs, &promoted_type, asm_data, stack_data, global_asm_data));
                unwrap_let!(DataType::RAW(BaseType::Scalar(promoted_underlying)) = &promoted_type);
                result.add_instruction(AsmOperation::MUL {
                    multiplier: RegOrMem::GPReg(GPRegister::secondary()),
                    data_type: promoted_underlying.clone()
                });

            },
            BinaryExpressionOperator::Divide => {
                result.add_comment("dividing numbers");

                result.merge(&put_lhs_ax_rhs_cx(&self.lhs, &promoted_type, &self.rhs, &promoted_type, asm_data, stack_data, global_asm_data));

                unwrap_let!(DataType::RAW(BaseType::Scalar(promoted_base)) = promoted_type);

                result.add_instruction(AsmOperation::DIV {
                    divisor: RegOrMem::GPReg(GPRegister::secondary()),
                    data_type: promoted_base
                });
            },

            BinaryExpressionOperator::Mod => {
                result.add_comment("calculating modulus");

                result.merge(&put_lhs_ax_rhs_cx(&self.lhs, &promoted_type, &self.rhs, &promoted_type, asm_data, stack_data, global_asm_data));

                unwrap_let!(DataType::RAW(BaseType::Scalar(promoted_base)) = promoted_type);

                result.add_instruction(AsmOperation::DIV {
                    divisor: RegOrMem::GPReg(GPRegister::secondary()),
                    data_type: promoted_base
                });

                //mod is returned in RDX
                result.add_instruction(AsmOperation::MOV { to: RegOrMem::GPReg(GPRegister::acc()), from: Operand::GPReg(GPRegister::_DX), size: promoted_size });
            }

            comparison if comparison.as_comparator_instr().is_some() => { // >, <, ==, >=, <=
                result.add_comment("comparing numbers");
                result.merge(&put_lhs_ax_rhs_cx(&self.lhs, &promoted_type, &self.rhs, &promoted_type, asm_data, stack_data, global_asm_data));

                let promoted_base = promoted_type.decay_to_primative();

                result.add_instruction(AsmOperation::CMP {
                    rhs: Operand::GPReg(GPRegister::secondary()),
                    data_type: promoted_base
                });

                let asm_comparison = comparison
                    .as_comparator_instr()
                    .unwrap()
                    .to_asm_comparison(match promoted_type.decay_to_primative() {
                        ScalarType::Float(_) => false,//float comparisons need unsigned setcc/jmpcc instructions for some reason
                        ScalarType::Integer(integer_type) => !integer_type.is_unsigned(),
                    });//take signedness and convert comparison kind to an asm comparison

                //create the correct setcc instruction
                result.add_instruction(AsmOperation::SETCC { comparison: asm_comparison });

            },

            operator if operator.as_boolean_instr().is_some() => {
                //warning: what if either side is not a boolean
                result.add_comment("applying boolean operator");

                result.merge(&put_lhs_ax_rhs_cx(&self.lhs, &DataType::RAW(BaseType::Scalar(ScalarType::Integer(IntegerType::_BOOL))), &self.rhs, &DataType::RAW(BaseType::Scalar(ScalarType::Integer(IntegerType::_BOOL))), asm_data, stack_data, global_asm_data));//casts too boolean

                let instruction = operator.as_boolean_instr().unwrap();

                result.add_instruction(AsmOperation::BitwiseOp {
                    secondary: Operand::GPReg(GPRegister::secondary()),
                    operation: instruction,
                });
            },

            operator if operator.as_bitwise_binary_instr().is_some() => {
                result.add_comment("applying bitwise operator");

                result.merge(&put_lhs_ax_rhs_cx(&self.lhs, &promoted_type, &self.rhs, &promoted_type, asm_data, stack_data, global_asm_data));

                let instruction = operator.as_bitwise_binary_instr().unwrap();

                result.add_instruction(AsmOperation::BitwiseOp {
                    secondary: Operand::GPReg(GPRegister::secondary()),
                    operation: instruction,
                });
            },

            //bit shifts left or right
            BinaryExpressionOperator::BitshiftRight => {
                result.add_comment("bitwise shift right");
                //lhs and rhs types are calculated individually as they do not influence each other
                let lhs_required_type = calculate_unary_type_arithmetic(&lhs_type);
                let rhs_required_type = DataType::RAW(BaseType::Scalar(ScalarType::Integer(IntegerType::U8)));//can only shift by u8 in assembly

                result.merge(&put_lhs_ax_rhs_cx(
                    &self.lhs, &lhs_required_type,
                    &self.rhs, &rhs_required_type,
                    asm_data, stack_data, global_asm_data
                ));
                
                unwrap_let!(DataType::RAW(lhs_base) = lhs_required_type);
                result.add_instruction(AsmOperation::SHR {
                    amount: Operand::GPReg(GPRegister::secondary()),
                    base_type: lhs_base
                });
            }
            BinaryExpressionOperator::BitshiftLeft => {
                result.add_comment("bitwise shift left");
                //lhs and rhs types are calculated individually as they do not influence each other
                let lhs_required_type = calculate_unary_type_arithmetic(&lhs_type);
                let rhs_required_type = DataType::RAW(BaseType::Scalar(ScalarType::Integer(IntegerType::U8)));//can only shift by u8 in assembly

                result.merge(&put_lhs_ax_rhs_cx(
                    &self.lhs, &lhs_required_type,
                    &self.rhs, &rhs_required_type,
                    asm_data, stack_data, global_asm_data
                ));
                
                unwrap_let!(DataType::RAW(lhs_base) = lhs_required_type);
                result.add_instruction(AsmOperation::SHL {
                    amount: Operand::GPReg(GPRegister::secondary()),
                    base_type: lhs_base
                });
            }

            BinaryExpressionOperator::AdditionCombination => {
                let promoted_type = ();
                let promoted_size = ();
                result.add_comment("addition combination (+=)");

                // *a++ += b is not the same as *a++ = *a++ + b because a might be incremented once or twice
                // so I need a custom implementation
                let lhs_type = self.lhs.accept(&mut GetDataTypeVisitor {asm_data});
                let rhs_type = self.rhs.accept(&mut GetDataTypeVisitor {asm_data});

                let lhs_ptr_asm = self.lhs.accept(&mut ReferenceVisitor {asm_data, stack_data, global_asm_data});
                result.merge(&lhs_ptr_asm);//put pointer to lhs in acc
                //save the pointer
                let lhs_ptr_temporary_address = stack_data.allocate(PTR_SIZE);
                result.add_instruction(AsmOperation::MOV {
                    to: RegOrMem::Mem(MemoryOperand::SubFromBP(lhs_ptr_temporary_address)),
                    from: Operand::GPReg(GPRegister::acc()),
                    size: PTR_SIZE
                });

                //calculate and cast rhs value
                let rhs_asm = self.rhs.accept(&mut ScalarInAccVisitor {asm_data, stack_data, global_asm_data});
                let rhs_cast_asm = cast_from_acc(&rhs_type, &lhs_type, asm_data);
                result.merge(&rhs_asm);
                result.merge(&rhs_cast_asm);//cast to lhs as that will be incremented

                if let DataType::POINTER(_) = lhs_type.decay() {
                    //you can only add pointer and number here, as per the C standard
                    let lhs_deref_size = lhs_type.remove_outer_modifier().memory_size(asm_data);

                    result.add_comment(format!("lhs is a pointer. make rhs {} times bigger", lhs_deref_size.size_bytes()));

                    //get the size of value pointed to by rhs
                    result.add_instruction(AsmOperation::MOV {
                        to: RegOrMem::GPReg(GPRegister::_CX),
                        from: Operand::Imm(lhs_deref_size.as_imm()),
                        size: MemorySize::from_bytes(8),
                    });

                    //multiply lhs by the size of value pointed to by rhs, so that +1 would skip along 1 value, not 1 byte
                    result.add_instruction(AsmOperation::MUL {
                        multiplier: RegOrMem::GPReg(GPRegister::_CX),
                        data_type: lhs_type.decay_to_primative(),//rhs has been promoted to lhs's type
                    });
                }

                //put RHS in secondary
                result.add_commented_instruction(
                    AsmOperation::MOV { to: RegOrMem::GPReg(GPRegister::secondary()), from: Operand::GPReg(GPRegister::acc()), size: MemorySize::from_bytes(8)},
                    "put RHS in secondary"
                );

                // put a pointer to lhs to acc
                result.add_commented_instruction(
                    AsmOperation::MOV { 
                        to: RegOrMem::GPReg(GPRegister::acc()),
                        from: Operand::Mem(MemoryOperand::SubFromBP(lhs_ptr_temporary_address)),
                        size: PTR_SIZE
                    },
                    "put a pointer to lhs in acc"
                );
                //dereference the pointer
                result.add_commented_instruction(
                    AsmOperation::MOV {
                    to: RegOrMem::GPReg(GPRegister::acc()), 
                        from: Operand::Mem(MemoryOperand::MemoryAddress { pointer_reg: GPRegister::acc()}),
                        size: lhs_type.memory_size(asm_data)
                    },
                    "dereference pointer to lhs"
                );
                //increment lhs by rhs
                result.add_commented_instruction(AsmOperation::ADD {
                    increment: Operand::GPReg(GPRegister::secondary()),
                    data_type: lhs_type.decay_to_primative()
                }, "increment lhs by rhs");
                //put the addition result in secondary
                result.add_commented_instruction(
                    AsmOperation::MOV { to: RegOrMem::GPReg(GPRegister::secondary()), from: Operand::GPReg(GPRegister::acc()), size: lhs_type.memory_size(asm_data) },
                    "move the result to secondary"
                );

                //get a pointer to lhs again
                result.add_commented_instruction(
                    AsmOperation::MOV { 
                        to: RegOrMem::GPReg(GPRegister::acc()),
                        from: Operand::Mem(MemoryOperand::SubFromBP(lhs_ptr_temporary_address)),
                        size: PTR_SIZE
                    },
                    "get a pointer to lhs again"
                );
                //save the results
                result.add_commented_instruction(
                    AsmOperation::MOV { to: RegOrMem::Mem(MemoryOperand::MemoryAddress { pointer_reg: GPRegister::acc() }), from: Operand::GPReg(GPRegister::secondary()), size: lhs_type.memory_size(asm_data) },
                    "save the results back to lhs"
                );

                result.add_commented_instruction(
                    AsmOperation::MOV { to: RegOrMem::GPReg(GPRegister::acc()), from: Operand::GPReg(GPRegister::secondary()), size: lhs_type.memory_size(asm_data) },
                    "leave the result in acc"
                );
            }

            BinaryExpressionOperator::SubtractionCombination => {
                let promoted_type = ();
                let promoted_size = ();
                result.add_comment("addition combination (+=)");

                // *a++ += b is not the same as *a++ = *a++ + b because a might be incremented once or twice
                // so I need a custom implementation
                let lhs_type = self.lhs.accept(&mut GetDataTypeVisitor {asm_data});
                let rhs_type = self.rhs.accept(&mut GetDataTypeVisitor {asm_data});

                let lhs_ptr_asm = self.lhs.accept(&mut ReferenceVisitor {asm_data, stack_data, global_asm_data});
                result.merge(&lhs_ptr_asm);//put pointer to lhs in acc
                //save the pointer
                let lhs_ptr_temporary_address = stack_data.allocate(PTR_SIZE);
                result.add_instruction(AsmOperation::MOV {
                    to: RegOrMem::Mem(MemoryOperand::SubFromBP(lhs_ptr_temporary_address)),
                    from: Operand::GPReg(GPRegister::acc()),
                    size: PTR_SIZE
                });

                //calculate and cast rhs value
                let rhs_asm = self.rhs.accept(&mut ScalarInAccVisitor {asm_data, stack_data, global_asm_data});
                let rhs_cast_asm = cast_from_acc(&rhs_type, &lhs_type, asm_data);
                result.merge(&rhs_asm);
                result.merge(&rhs_cast_asm);//cast to lhs as that will be incremented

                if let DataType::POINTER(_) = lhs_type.decay() {
                    //you can only add pointer and number here, as per the C standard
                    let lhs_deref_size = lhs_type.remove_outer_modifier().memory_size(asm_data);

                    result.add_comment(format!("lhs is a pointer. make rhs {} times bigger", lhs_deref_size.size_bytes()));

                    //get the size of value pointed to by rhs
                    result.add_instruction(AsmOperation::MOV {
                        to: RegOrMem::GPReg(GPRegister::_CX),
                        from: Operand::Imm(lhs_deref_size.as_imm()),
                        size: MemorySize::from_bytes(8),
                    });

                    //multiply lhs by the size of value pointed to by rhs, so that +1 would skip along 1 value, not 1 byte
                    result.add_instruction(AsmOperation::MUL {
                        multiplier: RegOrMem::GPReg(GPRegister::_CX),
                        data_type: lhs_type.decay_to_primative(),//rhs has been promoted to lhs's type
                    });
                    
                }

                //put RHS in secondary
                result.add_commented_instruction(
                    AsmOperation::MOV { to: RegOrMem::GPReg(GPRegister::secondary()), from: Operand::GPReg(GPRegister::acc()), size: MemorySize::from_bytes(8)},
                    "put RHS in secondary"
                );

                // put a pointer to lhs to acc
                result.add_commented_instruction(
                    AsmOperation::MOV { 
                        to: RegOrMem::GPReg(GPRegister::acc()),
                        from: Operand::Mem(MemoryOperand::SubFromBP(lhs_ptr_temporary_address)),
                        size: PTR_SIZE
                    },
                    "put a pointer to lhs in acc"
                );
                //dereference the pointer
                result.add_commented_instruction(
                    AsmOperation::MOV {
                    to: RegOrMem::GPReg(GPRegister::acc()), 
                        from: Operand::Mem(MemoryOperand::MemoryAddress { pointer_reg: GPRegister::acc()}),
                        size: lhs_type.memory_size(asm_data)
                    },
                    "dereference pointer to lhs"
                );
                //increment lhs by rhs
                result.add_commented_instruction(AsmOperation::SUB {
                    decrement: Operand::GPReg(GPRegister::secondary()),
                    data_type: lhs_type.decay_to_primative()
                }, "increment lhs by rhs");
                //put the addition result in secondary
                result.add_commented_instruction(
                    AsmOperation::MOV { to: RegOrMem::GPReg(GPRegister::secondary()), from: Operand::GPReg(GPRegister::acc()), size: lhs_type.memory_size(asm_data) },
                    "move the result to secondary"
                );

                //get a pointer to lhs again
                result.add_commented_instruction(
                    AsmOperation::MOV { 
                        to: RegOrMem::GPReg(GPRegister::acc()),
                        from: Operand::Mem(MemoryOperand::SubFromBP(lhs_ptr_temporary_address)),
                        size: PTR_SIZE
                    },
                    "get a pointer to lhs again"
                );
                //save the results
                result.add_commented_instruction(
                    AsmOperation::MOV { to: RegOrMem::Mem(MemoryOperand::MemoryAddress { pointer_reg: GPRegister::acc() }), from: Operand::GPReg(GPRegister::secondary()), size: lhs_type.memory_size(asm_data) },
                    "save the results back to lhs"
                );

                result.add_commented_instruction(
                    AsmOperation::MOV { to: RegOrMem::GPReg(GPRegister::acc()), from: Operand::GPReg(GPRegister::secondary()), size: lhs_type.memory_size(asm_data) },
                    "leave the result in acc"
                );
            }

            _ => panic!("assignment must be done beforehand")
        }

        result
    }

    pub fn new(lhs: Expression, operator: BinaryExpressionOperator, rhs: Expression) -> BinaryExpression {
        BinaryExpression {
            lhs: Box::new(lhs),
            operator,
            rhs: Box::new(rhs),
        }
    }

    pub fn lhs(&self) -> &Expression {
        &self.lhs
    }
    pub fn rhs(&self) -> &Expression {
        &self.rhs
    }
    pub fn operator(&self) -> &BinaryExpressionOperator {
        &self.operator
    }
}

impl ASTDisplay for BinaryExpression {
    fn display_ast(&self, f: &mut crate::debugging::TreeDisplayInfo) {
        let operator: &str = self.operator.clone().into();
        f.write(&operator.yellow().to_string());
        f.indent();
        self.lhs.display_ast(f);
        self.rhs.display_ast(f);
        f.dedent();
    }
}

fn apply_pointer_scaling(lhs_casted: &StackItemKey, lhs_type: &DataType, rhs_casted: &StackItemKey, rhs_type: &DataType, promoted_type: &DataType, asm_data: &AsmData, stack_data: &mut SimpleStackFrame, global_asm_data: &mut GlobalAsmData) -> (Assembly, StackItemKey, StackItemKey) {
    let mut result = Assembly::make_empty();

    //decay pointers to u64, so that some things are less ambiguous?
    let promoted_primative = promoted_type.decay_to_primative();
    let promoted_size = promoted_primative.memory_size();

    let lhs_location = stack_data.allocate(promoted_size);
    let rhs_location = stack_data.allocate(promoted_size);

    //promote lhs and rhs
    result.add_instruction(AsmOperation::CAST { from: Storage::Stack(lhs_raw_location.unwrap()), from_type: lhs_type.clone(), to: Storage::Stack(lhs_location), to_type: DataType::RAW(BaseType::Scalar(promoted_primative.clone())) });
    let (rhs_asm, rhs_raw_location) = rhs.generate_ir(asm_data, stack_data, global_asm_data);
    result.merge(&rhs_asm);
    result.add_instruction(AsmOperation::CAST { from: Storage::Stack(rhs_raw_location.unwrap()), from_type: rhs_type.clone(), to: Storage::Stack(rhs_location), to_type: DataType::RAW(BaseType::Scalar(promoted_primative.clone())) });

    if let DataType::POINTER(rhs_pointed_at) = &rhs_type {
        let rhs_deref_size = rhs_pointed_at.memory_size(asm_data);
        result.add_commented_instruction(AsmOperation::MUL {
            lhs: Storage::Constant(rhs_deref_size.as_imm()),
            rhs: Storage::Stack(lhs_location),
            to: Storage::Stack(lhs_location),
            data_type: promoted_primative.clone(),
        }, format!("rhs is a pointer. make lhs {} times bigger", rhs_deref_size.size_bytes()));
    }

    if let DataType::POINTER(lhs_pointed_at) = &lhs_type {
        let lhs_deref_size = lhs_pointed_at.memory_size(asm_data);
        result.add_commented_instruction(AsmOperation::MUL {
            lhs: Storage::Constant(lhs_deref_size.as_imm()),
            rhs: Storage::Stack(rhs_location),
            to: Storage::Stack(rhs_location),
            data_type: promoted_primative,
        }, format!("rhs is a pointer. make lhs {} times bigger", lhs_deref_size.size_bytes()));
    }

    (result, lhs_location, rhs_location)
}