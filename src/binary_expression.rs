
use colored::Colorize;
use unwrap_let::unwrap_let;
use memory_size::MemorySize;
use crate::{asm_boilerplate::cast_from_acc, asm_gen_data::AsmData, assembly::{assembly::Assembly, operand::{immediate::MemorySizeExt, memory_operand::MemoryOperand, register::GPRegister, Operand, RegOrMem}, operation::AsmOperation}, data_type::{base_type::{BaseType, IntegerType, ScalarType}, recursive_data_type::{calculate_promoted_type_arithmetic, calculate_unary_type_arithmetic, DataType}}, debugging::ASTDisplay, expression::{binary_expression_operator::BinaryExpressionOperator, expression::{generate_assembly_for_assignment, put_lhs_ax_rhs_cx, Expression}}, expression_visitors::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor, put_scalar_in_acc::ScalarInAccVisitor}, stack_allocation::StackAllocator};

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
    
    pub fn generate_assembly(&self, asm_data: &AsmData, stack_data: &mut StackAllocator) -> Assembly {
        let mut result = Assembly::make_empty();

        if self.operator == BinaryExpressionOperator::Assign {
            return generate_assembly_for_assignment(&self.lhs, &self.rhs, asm_data, stack_data);
        }

        let lhs_type = self.lhs.accept(&mut GetDataTypeVisitor {asm_data});
        let rhs_type = self.rhs.accept(&mut GetDataTypeVisitor {asm_data});

        let promoted_type = match &self.operator {//I already have a function for this?
            BinaryExpressionOperator::Assign => panic!("assignment already done"),
            x if x.as_boolean_instr().is_some() => DataType::RAW(BaseType::Scalar(ScalarType::Integer(IntegerType::_BOOL))),//is a boolean operator, operands are booleans
            _ => calculate_promoted_type_arithmetic(&lhs_type, &rhs_type)//else find a common meeting ground
        };
        let promoted_size = promoted_type.memory_size(asm_data);

        match &self.operator {
            BinaryExpressionOperator::Add => {
                result.add_comment(format!("adding {} numbers", promoted_size));

                result.merge(&apply_pointer_scaling(&self.lhs, &self.rhs, &promoted_type, asm_data, stack_data));

                result.add_instruction(AsmOperation::ADD {
                    increment: Operand::GPReg(GPRegister::secondary()),
                    data_type: promoted_type.decay_to_primative()
                });

                //result is now in AX
                
            },
            BinaryExpressionOperator::Subtract => {
                result.add_comment(format!("subtracting {} numbers", promoted_size));

                result.merge(&apply_pointer_scaling(&self.lhs, &self.rhs, &promoted_type, asm_data, stack_data));

                result.add_instruction(AsmOperation::SUB {
                    decrement: Operand::GPReg(GPRegister::secondary()),
                    data_type: promoted_type.decay_to_primative()
                });

                //result is now in AX

            }
            BinaryExpressionOperator::Multiply => {
                result.add_comment("mulitplying numbers");

                result.merge(&put_lhs_ax_rhs_cx(&self.lhs, &promoted_type, &self.rhs, &promoted_type, asm_data, stack_data));

                unwrap_let!(DataType::RAW(promoted_underlying) = &promoted_type);
                assert!(promoted_underlying.is_integer());//floating point multiply??

                result.add_instruction(AsmOperation::MUL {
                    multiplier: RegOrMem::GPReg(GPRegister::secondary()),
                    data_type: promoted_type
                });

            },
            BinaryExpressionOperator::Divide => {
                result.add_comment("dividing numbers");

                result.merge(&put_lhs_ax_rhs_cx(&self.lhs, &promoted_type, &self.rhs, &promoted_type, asm_data, stack_data));

                unwrap_let!(DataType::RAW(BaseType::Scalar(promoted_base)) = promoted_type);

                result.add_instruction(AsmOperation::DIV {
                    divisor: RegOrMem::GPReg(GPRegister::secondary()),
                    data_type: promoted_base
                });
            },

            BinaryExpressionOperator::Mod => {
                result.add_comment("calculating modulus");

                result.merge(&put_lhs_ax_rhs_cx(&self.lhs, &promoted_type, &self.rhs, &promoted_type, asm_data, stack_data));

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
                result.merge(&put_lhs_ax_rhs_cx(&self.lhs, &promoted_type, &self.rhs, &promoted_type, asm_data, stack_data));

                let promoted_base = promoted_type.decay_to_primative();

                result.add_instruction(AsmOperation::CMP {
                    rhs: Operand::GPReg(GPRegister::secondary()),
                    data_type: promoted_base
                });

                let asm_comparison = comparison
                    .as_comparator_instr()
                    .unwrap()
                    .to_asm_comparison(match promoted_type.decay_to_primative() {
                        ScalarType::Float(_) => true,//float is always signed, but either signed or unsigned instructions both work
                        ScalarType::Integer(integer_type) => !integer_type.is_unsigned(),
                    });//take signedness and convert comparison kind to an asm comparison

                //create the correct setcc instruction
                result.add_instruction(AsmOperation::SETCC { comparison: asm_comparison });

            },

            operator if operator.as_boolean_instr().is_some() => {
                //warning: what if either side is not a boolean
                result.add_comment("applying boolean operator");

                result.merge(&put_lhs_ax_rhs_cx(&self.lhs, &DataType::RAW(BaseType::Scalar(ScalarType::Integer(IntegerType::_BOOL))), &self.rhs, &DataType::RAW(BaseType::Scalar(ScalarType::Integer(IntegerType::_BOOL))), asm_data, stack_data));//casts too boolean

                let instruction = operator.as_boolean_instr().unwrap();

                result.add_instruction(AsmOperation::BitwiseOp {
                    secondary: Operand::GPReg(GPRegister::secondary()),
                    operation: instruction,
                });
            },

            operator if operator.as_bitwise_binary_instr().is_some() => {
                result.add_comment("applying bitwise operator");

                result.merge(&put_lhs_ax_rhs_cx(&self.lhs, &promoted_type, &self.rhs, &promoted_type, asm_data, stack_data));

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
                    asm_data, stack_data
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
                    asm_data, stack_data
                ));
                
                unwrap_let!(DataType::RAW(lhs_base) = lhs_required_type);
                result.add_instruction(AsmOperation::SHL {
                    amount: Operand::GPReg(GPRegister::secondary()),
                    base_type: lhs_base
                });
            }

            _ => panic!("operator to binary expression is invalid")
        }

        result
    }

    pub fn get_data_type(&self, asm_data: &AsmData) -> DataType {
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
                    &self.lhs.accept(&mut GetDataTypeVisitor { asm_data }),//type of lhs
                    &self.rhs.accept(&mut GetDataTypeVisitor { asm_data }),//type of rhs
                )
            },

            BinaryExpressionOperator::Assign => self.lhs.accept(&mut GetDataTypeVisitor {asm_data}),//assigning, rhs must be converted to lhs

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

fn apply_pointer_scaling(lhs: &Expression, rhs: &Expression, promoted_type: &DataType,  asm_data: &AsmData, stack_data: &mut StackAllocator) -> Assembly {
    let mut result = Assembly::make_empty();

    let lhs_type = lhs.accept(&mut GetDataTypeVisitor {asm_data});
    let rhs_type = rhs.accept(&mut GetDataTypeVisitor {asm_data});
    let promoted_size = promoted_type.memory_size(asm_data);

    let lhs_asm = lhs.accept(&mut ScalarInAccVisitor {asm_data, stack_data});
    let lhs_cast_asm = cast_from_acc(&lhs_type, &promoted_type, asm_data);
    result.merge(&lhs_asm);//put lhs in acc
    result.merge(&lhs_cast_asm);//cast to the correct type

    if let DataType::POINTER(_) = rhs.accept(&mut GetDataTypeVisitor {asm_data}).decay() {//adding array or pointer to int
        //you can only add pointer and number here, as per the C standard

        let rhs_deref_size = rhs_type.remove_outer_modifier().memory_size(asm_data);
        result.add_comment(format!("rhs is a pointer. make lhs {} times bigger", rhs_deref_size.size_bytes()));

        assert!(promoted_type.memory_size(asm_data).size_bytes() == 8);
        //get the size of value pointed to by rhs
        result.add_instruction(AsmOperation::MOV {
            to: RegOrMem::GPReg(GPRegister::_CX),
            from: Operand::Imm(rhs_deref_size.as_imm()),
            size: MemorySize::from_bytes(8),
        });

        //multiply lhs by the size of value pointed to by rhs, so that +1 would skip along 1 value, not 1 byte
        result.add_instruction(AsmOperation::MUL {
            multiplier: RegOrMem::GPReg(GPRegister::_CX),
            data_type: promoted_type.clone(),
        });
        
        //lhs is now in AX
    }

    //save lhs to stack, as preprocessing for it is done
    //allocate temporary lhs storage
    let lhs_temporary_address = stack_data.allocate(promoted_size);
    result.add_instruction(AsmOperation::MOV {
        to: RegOrMem::Mem(MemoryOperand::SubFromBP(lhs_temporary_address)),
        from: Operand::GPReg(GPRegister::acc()),
        size: promoted_size
    });

    //calculate and cast rhs value
    let rhs_asm = rhs.accept(&mut ScalarInAccVisitor {asm_data, stack_data});
    let rhs_cast_asm = cast_from_acc(&rhs_type, &promoted_type, asm_data);
    result.merge(&rhs_asm);
    result.merge(&rhs_cast_asm);


    if let DataType::POINTER(_) = lhs.accept(&mut GetDataTypeVisitor {asm_data}).decay() {
        //you can only add pointer and number here, as per the C standard
        let lhs_deref_size = lhs_type.remove_outer_modifier().memory_size(asm_data);

        result.add_comment(format!("lhs is a pointer. make rhs {} times bigger", lhs_deref_size.size_bytes()));

        assert!(promoted_type.memory_size(asm_data).size_bytes() == 8);
        //get the size of value pointed to by rhs
        result.add_instruction(AsmOperation::MOV {
            to: RegOrMem::GPReg(GPRegister::_CX),
            from: Operand::Imm(lhs_deref_size.as_imm()),
            size: MemorySize::from_bytes(8),
        });

        //multiply lhs by the size of value pointed to by rhs, so that +1 would skip along 1 value, not 1 byte
        result.add_instruction(AsmOperation::MUL {
            multiplier: RegOrMem::GPReg(GPRegister::_CX),
            data_type: promoted_type.clone(),
        });
    }

    //put RHS in CX 
    result.add_instruction(AsmOperation::MOV { to: RegOrMem::GPReg(GPRegister::secondary()), from: Operand::GPReg(GPRegister::acc()), size: MemorySize::from_bytes(8)});

    //read lhs to acc
    result.add_instruction(AsmOperation::MOV { to: RegOrMem::GPReg(GPRegister::acc()), from: Operand::Mem(MemoryOperand::SubFromBP(lhs_temporary_address)), size: promoted_size });

    result
}