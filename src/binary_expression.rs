
use colored::Colorize;
use stack_management::{simple_stack_frame::SimpleStackFrame, stack_item::StackItemKey};
use unwrap_let::unwrap_let;
use memory_size::MemorySize;
use crate::{asm_gen_data::{AsmData, GlobalAsmData}, assembly::{assembly::Assembly, operand::{immediate::ToImmediate, memory_operand::MemoryOperand, register::GPRegister, Operand, Storage, PTR_SIZE}, operation::AsmOperation}, data_type::{base_type::{BaseType, IntegerType, ScalarType}, recursive_data_type::{calculate_promoted_type_arithmetic, calculate_unary_type_arithmetic, DataType}}, debugging::ASTDisplay, expression::{binary_expression_operator::BinaryExpressionOperator, expression::{generate_assembly_for_assignment, promote, Expression}}, expression_visitors::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor, reference_assembly_visitor::ReferenceVisitor}, generate_ir::{GenerateIR, GetType}, number_literal::typed_value::NumberLiteral};

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

    pub fn new(lhs: Expression, operator: BinaryExpressionOperator, rhs: Expression) -> BinaryExpression {
        BinaryExpression {
            lhs: Box::new(lhs),
            operator,
            rhs: Box::new(rhs),
        }
    }
}
impl GetType for BinaryExpression {
    fn get_type(&self, asm_data: &AsmData) -> DataType {
        match self.operator {
            BinaryExpressionOperator::BitwiseOr |
            BinaryExpressionOperator::BitwiseAnd |
            BinaryExpressionOperator::BitwiseXor |
            BinaryExpressionOperator::Add |
            BinaryExpressionOperator::Subtract |
            BinaryExpressionOperator::Multiply | 
            BinaryExpressionOperator::Divide | 
            BinaryExpressionOperator::Mod => {
                calculate_promoted_type_arithmetic(//calculate type when data types:
                    &self.lhs.accept(&mut GetDataTypeVisitor { asm_data}),//type of lhs
                    &self.rhs.accept(&mut GetDataTypeVisitor { asm_data}),//type of rhs
                )
            },

            BinaryExpressionOperator::Assign |
            BinaryExpressionOperator::AdditionCombination |
            BinaryExpressionOperator::SubtractionCombination => self.lhs.accept(&mut GetDataTypeVisitor {asm_data}),//assigning, rhs must be converted to lhs

            //bit shifts have lhs promoted, then resultant type is the same as promoted lhs
            BinaryExpressionOperator::BitshiftLeft |
            BinaryExpressionOperator::BitshiftRight => calculate_unary_type_arithmetic(&self.lhs.accept(&mut GetDataTypeVisitor {asm_data})),

            BinaryExpressionOperator::CmpLess |
            BinaryExpressionOperator::CmpGreater |
            BinaryExpressionOperator::CmpGreaterEqual |
            BinaryExpressionOperator::CmpLessEqual |
            BinaryExpressionOperator::CmpEqual |
            BinaryExpressionOperator::CmpNotEqual |
            BinaryExpressionOperator::BooleanOr |
            BinaryExpressionOperator::BooleanAnd  => DataType::RAW(BaseType::Scalar(ScalarType::Integer(IntegerType::_BOOL))),
        }
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
        
        //what type the result is
        let resultant_type = self.get_type(asm_data);

        //the type lhs and rhs have to be promoted to (sometimes rhs doesn't get promoted to this, as in bit shifts)
        let promoted_type = match &self.operator {
            BinaryExpressionOperator::Assign => unreachable!(),

            BinaryExpressionOperator::BitshiftLeft |
            BinaryExpressionOperator::BitshiftRight => calculate_unary_type_arithmetic(&lhs_type),//bit shift type is related to the number being shifted
            x if x.as_boolean_instr().is_some() => DataType::RAW(BaseType::Scalar(ScalarType::Integer(IntegerType::_BOOL))),//is a boolean operator, operands are booleans
            BinaryExpressionOperator::AdditionCombination => lhs_type,
            _ => calculate_promoted_type_arithmetic(&lhs_type, &rhs_type)//else find a common meeting ground
        };
        
        //where the result of the computation goes
        let resultant_location = stack_data.allocate(resultant_type.memory_size(asm_data));
        //use these as lhs and rhs, as they have correct promotions (or correctly have no promotion)
        let (lhs_promoted, rhs_promoted) = match &self.operator {
            BinaryExpressionOperator::Add |
            BinaryExpressionOperator::Subtract |
            BinaryExpressionOperator::Multiply |
            BinaryExpressionOperator::Divide |
            BinaryExpressionOperator::Mod |
            BinaryExpressionOperator::CmpEqual |
            BinaryExpressionOperator::CmpGreater |
            BinaryExpressionOperator::CmpGreaterEqual |
            BinaryExpressionOperator::CmpLess |
            BinaryExpressionOperator::CmpLessEqual |
            BinaryExpressionOperator::CmpNotEqual |
            BinaryExpressionOperator::BooleanOr |
            BinaryExpressionOperator::BooleanAnd |
            BinaryExpressionOperator::AdditionCombination => {
                let (promote_lhs_op, lhs_promoted) = promote(lhs_result.unwrap(), lhs_type, promoted_type, stack_data, asm_data);
                let (promote_rhs_op, rhs_promoted) = promote(rhs_result.unwrap(), rhs_type, promoted_type, stack_data, asm_data);

                result.add_instruction(promote_lhs_op);
                result.add_instruction(promote_rhs_op);

                (lhs_promoted, rhs_promoted)
            }

            BinaryExpressionOperator::BitshiftLeft |
            BinaryExpressionOperator::BitshiftRight => {
                let (promote_lhs_op, lhs_promoted) = promote(lhs_result.unwrap(), lhs_type, promoted_type, stack_data, asm_data);
                let (promote_rhs_op, rhs_promoted) = promote(rhs_result.unwrap(), rhs_type, DataType::RAW(BaseType::Scalar(ScalarType::Integer(IntegerType::U8))), stack_data, asm_data);//can only shift by u8 in assembly

                result.add_instruction(promote_lhs_op);
                result.add_instruction(promote_rhs_op);

                (lhs_promoted, rhs_promoted)
            }



            _ => todo!()
        };

        match &self.operator {
            BinaryExpressionOperator::Add => {
                result.add_comment(format!("adding {} numbers", promoted_type.memory_size(asm_data)));

                let (ptr_scale_asm, lhs_scaled, rhs_scaled) = apply_pointer_scaling(lhs_promoted, &lhs_type, rhs_promoted, &rhs_type, &promoted_type, asm_data);
                result.merge(&ptr_scale_asm);

                result.add_instruction(AsmOperation::ADD {
                    lhs: Storage::Stack(lhs_scaled),
                    rhs: Storage::Stack(rhs_scaled),
                    to: Storage::Stack(resultant_location),
                    data_type: promoted_type.decay_to_primative(),
                });
                
            },
            BinaryExpressionOperator::Subtract => {
                result.add_comment(format!("subtracting {} numbers", promoted_type.memory_size(asm_data)));

                let (ptr_scale_asm, lhs_scaled, rhs_scaled) = apply_pointer_scaling(lhs_promoted, &lhs_type, rhs_promoted, &rhs_type, &promoted_type, asm_data);
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

                unwrap_let!(DataType::RAW(BaseType::Scalar(promoted_underlying)) = promoted_type);
                result.add_instruction(AsmOperation::MUL {
                    data_type: promoted_underlying,
                    lhs: Storage::Stack(lhs_promoted),
                    rhs: Storage::Stack(rhs_promoted),
                    to: Storage::Stack(resultant_location),
                });

            },
            BinaryExpressionOperator::Divide => {
                result.add_comment("dividing numbers");

                unwrap_let!(DataType::RAW(BaseType::Scalar(promoted_base)) = promoted_type);
                result.add_instruction(AsmOperation::DIV {
                    data_type: promoted_base,
                    lhs: Storage::Stack(lhs_promoted),
                    rhs: Storage::Stack(rhs_promoted),
                    to: Storage::Stack(resultant_location),
                });
            },

            BinaryExpressionOperator::Mod => {
                result.add_comment("calculating modulus");

                unwrap_let!(DataType::RAW(BaseType::Scalar(ScalarType::Integer(promoted_base))) = promoted_type);
                result.add_instruction(AsmOperation::MOD {
                    data_type: promoted_base,
                    lhs: Storage::Stack(lhs_promoted),
                    rhs: Storage::Stack(rhs_promoted),
                    to: Storage::Stack(resultant_location),
                });
            }

            comparison if comparison.as_comparator_instr().is_some() => { // >, <, ==, >=, <=
                result.add_comment("comparing numbers");

                let promoted_base = promoted_type.decay_to_primative();
                result.add_instruction(AsmOperation::CMP {
                    lhs: Storage::Stack(lhs_promoted),
                    rhs: Storage::Stack(rhs_promoted),
                    data_type: promoted_base,
                });

                let asm_comparison = comparison
                    .as_comparator_instr()
                    .unwrap()
                    .to_asm_comparison(match promoted_type.decay_to_primative() {
                        ScalarType::Float(_) => false,//float comparisons need unsigned setcc/jmpcc instructions for some reason
                        ScalarType::Integer(integer_type) => !integer_type.is_unsigned(),
                    });//take signedness and convert comparison kind to an asm comparison

                //create the correct setcc instruction
                result.add_instruction(AsmOperation::SETCC {
                    comparison: asm_comparison,
                    to: Storage::Stack(resultant_location),
                    data_type: ScalarType::Integer(IntegerType::_BOOL)//returns a bool, not taking into account the promoted type of lhs and rhs (obviously)
                });

            },

            operator if operator.as_boolean_instr().is_some() => {
                assert_eq!(promoted_type, DataType::RAW(BaseType::Scalar(ScalarType::Integer(IntegerType::_BOOL))));
                result.add_comment("applying boolean operator");

                let instruction = operator.as_boolean_instr().unwrap();
                //operands should have been converted into booleans as this is a boolean instruction
                result.add_instruction(AsmOperation::BitwiseOp {
                    operation: instruction,
                    lhs: Storage::Stack(lhs_promoted),
                    rhs: Storage::Stack(rhs_promoted),
                    to: Storage::Stack(resultant_location),
                    size: promoted_type.memory_size(asm_data),
                });
            },

            operator if operator.as_bitwise_binary_instr().is_some() => {
                result.add_comment("applying bitwise operator");

                let instruction = operator.as_bitwise_binary_instr().unwrap();

                result.add_instruction(AsmOperation::BitwiseOp {
                    operation: instruction,
                    lhs: Storage::Stack(lhs_promoted),
                    rhs: Storage::Stack(rhs_promoted),
                    to: Storage::Stack(resultant_location),
                    size: promoted_type.memory_size(asm_data),
                });
            },

            //bit shifts left or right
            BinaryExpressionOperator::BitshiftRight => {
                result.add_comment("bitwise shift right");
                
                unwrap_let!(DataType::RAW(BaseType::Scalar(ScalarType::Integer(lhs_type))) = promoted_type);
                result.add_instruction(AsmOperation::SHR {
                    from: Storage::Stack(lhs_promoted),
                    from_type: lhs_type,
                    amount: Storage::Stack(rhs_promoted),
                    to: Storage::Stack(resultant_location),
                });
            }
            BinaryExpressionOperator::BitshiftLeft => {
                result.add_comment("bitwise shift left");
                
                unwrap_let!(DataType::RAW(BaseType::Scalar(ScalarType::Integer(lhs_type))) = promoted_type);
                result.add_instruction(AsmOperation::SHL {
                    from: Storage::Stack(lhs_promoted),
                    from_type: lhs_type,
                    amount: Storage::Stack(rhs_promoted),
                    to: Storage::Stack(resultant_location),
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

fn apply_pointer_scaling(lhs_promoted: StackItemKey, raw_lhs_type: &DataType, rhs_promoted: StackItemKey, raw_rhs_type: &DataType, promoted_type: &DataType, asm_data: &AsmData) -> (Assembly, StackItemKey, StackItemKey) {
    let mut result = Assembly::make_empty();

    //decay pointers to u64, so that some things are less ambiguous?
    let promoted_primative = promoted_type.decay_to_primative();

    if let DataType::POINTER(rhs_pointed_at) = &raw_rhs_type {
        let rhs_deref_size = rhs_pointed_at.memory_size(asm_data);
        result.add_commented_instruction(AsmOperation::MUL {
            lhs: Storage::Constant(rhs_deref_size.as_imm()),
            rhs: Storage::Stack(lhs_promoted),
            to: Storage::Stack(lhs_promoted),
            data_type: promoted_primative.clone(),
        }, format!("rhs is a pointer. make lhs {} times bigger", rhs_deref_size.size_bytes()));
    }

    if let DataType::POINTER(lhs_pointed_at) = &raw_lhs_type {
        let lhs_deref_size = lhs_pointed_at.memory_size(asm_data);
        result.add_commented_instruction(AsmOperation::MUL {
            lhs: Storage::Constant(lhs_deref_size.as_imm()),
            rhs: Storage::Stack(rhs_promoted),
            to: Storage::Stack(rhs_promoted),
            data_type: promoted_primative,
        }, format!("rhs is a pointer. make lhs {} times bigger", lhs_deref_size.size_bytes()));
    }

    (result, lhs_promoted, rhs_promoted)
}