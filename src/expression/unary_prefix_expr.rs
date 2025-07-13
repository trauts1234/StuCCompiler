use crate::{asm_boilerplate::{cast_from_acc, cast_raw_from_acc}, asm_gen_data::AsmData, assembly::{assembly::Assembly, comparison::AsmComparison, operand::{immediate::{ImmediateValue, MemorySizeExt}, memory_operand::MemoryOperand, register::GPRegister, Operand, RegOrMem, PTR_SIZE}, operation::AsmOperation}, data_type::{base_type::{BaseType, IntegerType, ScalarType}, recursive_data_type::{calculate_unary_type_arithmetic, DataType}, type_modifier::DeclModifier}, debugging::ASTDisplay, expression::{expression::Expression, unary_prefix_operator::UnaryPrefixOperator}, expression_visitors::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor, put_scalar_in_acc::ScalarInAccVisitor, reference_assembly_visitor::ReferenceVisitor}};
use colored::Colorize;
use memory_size::MemorySize;
use unwrap_let::unwrap_let;

#[derive(Clone, Debug)]
pub struct UnaryPrefixExpression {
    operand: Box<Expression>,
    operator: UnaryPrefixOperator,
}

impl UnaryPrefixExpression {
    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_unary_prefix(self)
    }
    
    pub fn generate_assembly(&self, asm_data: &AsmData, stack_data: &mut MemorySize) -> Assembly {
        let mut result = Assembly::make_empty();

        match self.operator {
            UnaryPrefixOperator::Reference => {
                result.add_comment("getting address of something");
                //put address of the right hand side in acc
                let operand_ref_asm = self.operand.accept(&mut ReferenceVisitor {asm_data, stack_data});
                result.merge(&operand_ref_asm);
            },
            UnaryPrefixOperator::Dereference => {
                result.add_comment("dereferencing pointer");
                // put the address pointed to in rax
                let operand_asm = self.operand.accept(&mut ScalarInAccVisitor {asm_data, stack_data});
                result.merge(&operand_asm);

                if let DataType::ARRAY {..} = self.get_data_type(asm_data) {
                    //dereferencing results in an array, so I leave the address in RAX for future indexing etc.
                } else {
                    result.add_instruction(AsmOperation::MOV {
                        to: RegOrMem::GPReg(GPRegister::acc()),
                        from: Operand::Mem(MemoryOperand::MemoryAddress { pointer_reg: GPRegister::acc() }),
                        size: PTR_SIZE
                    });//dereference pointer
                }
            },
            UnaryPrefixOperator::Negate => {
                result.add_comment("negating something");

                let promoted_type = self.get_data_type(asm_data);
                let original_type = self.operand.accept(&mut GetDataTypeVisitor {asm_data});

                let operand_asm = self.operand.accept(&mut ScalarInAccVisitor {asm_data, stack_data});
                let cast_asm = cast_from_acc(&original_type, &promoted_type, asm_data);
                result.merge(&operand_asm);
                result.merge(&cast_asm);//cast to the correct type

                unwrap_let!(DataType::RAW(BaseType::Scalar(promoted_base)) = promoted_type);
                result.add_instruction(AsmOperation::NEG { data_type: promoted_base });//negate the promoted value
            },
            UnaryPrefixOperator::UnaryPlus => {
                result.add_comment("unary +");

                let promoted_type = self.get_data_type(asm_data);
                let original_type = self.operand.accept(&mut GetDataTypeVisitor {asm_data});

                let operand_asm = self.operand.accept(&mut ScalarInAccVisitor {asm_data, stack_data});
                let cast_asm = cast_from_acc(&original_type, &promoted_type, asm_data);
                result.merge(&operand_asm);
                result.merge(&cast_asm);//promote the type
                
            },
            UnaryPrefixOperator::Increment => {

                let promoted_type = self.get_data_type(asm_data);
                let original_type = self.operand.accept(&mut GetDataTypeVisitor {asm_data});

                let increment_amount = match &original_type {
                    DataType::UNKNOWNSIZEARRAY { .. } |
                    DataType::ARRAY {..} => panic!("this operation is invalid for arrays"),
                    
                    DataType::POINTER(underlying) => underlying.memory_size(asm_data).as_imm(),//increment pointer adds number of bytes
                    DataType::RAW(_) => ImmediateValue("1".to_string())
                };

                //push &self.operand
                let operand_asm = self.operand.accept(&mut ReferenceVisitor {asm_data, stack_data});
                result.merge(&operand_asm);
                *stack_data += PTR_SIZE;//allocate temporary lhs storage
                let operand_address_storage = stack_data.clone();
                result.add_instruction(AsmOperation::MOV {
                    to: RegOrMem::Mem(MemoryOperand::SubFromBP(operand_address_storage)),
                    from: Operand::GPReg(GPRegister::acc()),
                    size: PTR_SIZE
                });

                //put self.operand in acc
                let operand_asm = self.operand.accept(&mut ScalarInAccVisitor {asm_data, stack_data});
                result.merge(&operand_asm);

                //increment self.operand (in acc) as original type, so that it can be stored correctly afterwards
                result.add_instruction(AsmOperation::ADD { increment: Operand::Imm(increment_amount), data_type: original_type.clone() });

                //pop &self.operand to RCX
                result.add_instruction(AsmOperation::MOV {
                    to: RegOrMem::GPReg(GPRegister::secondary()),
                    from: Operand::Mem(MemoryOperand::SubFromBP(operand_address_storage)),
                    size: PTR_SIZE
                });

                //save the new value of self.operand
                result.add_instruction(AsmOperation::MOV {
                    to: RegOrMem::Mem(MemoryOperand::MemoryAddress { pointer_reg: GPRegister::secondary() }),
                    from: Operand::GPReg(GPRegister::acc()),
                    size: original_type.memory_size(asm_data)
                });

                let cast_asm = cast_from_acc(&original_type, &promoted_type, asm_data);//cast to the correct type
                result.merge(&cast_asm);

            }, 

            UnaryPrefixOperator::Decrement => {
                //TODO this code is duplicated from PLUSPLUS

                let promoted_type = self.get_data_type(asm_data);
                let original_type = self.operand.accept(&mut GetDataTypeVisitor {asm_data});

                let increment_amount = match &original_type {
                    DataType::UNKNOWNSIZEARRAY { .. } |
                    DataType::ARRAY {..} => panic!("this operation is invalid for arrays"),

                    DataType::POINTER(underlying) => underlying.memory_size(asm_data).as_imm(),//decrement by number of bytes
                    DataType::RAW(_) => ImmediateValue("1".to_string())
                };

                //push &self.operand
                let operand_asm = self.operand.accept(&mut ReferenceVisitor {asm_data, stack_data});
                result.merge(&operand_asm);
                *stack_data += PTR_SIZE;//allocate temporary lhs storage
                let operand_address_storage = stack_data.clone();
                result.add_instruction(AsmOperation::MOV {
                    to: RegOrMem::Mem(MemoryOperand::SubFromBP(operand_address_storage)),
                    from: Operand::GPReg(GPRegister::acc()),
                    size: PTR_SIZE
                });

                //put self.operand in acc
                let operand_asm = self.operand.accept(&mut ScalarInAccVisitor {asm_data, stack_data});
                result.merge(&operand_asm);

                //decrement self.operand (in acc) as original type, so that it can be stored correctly afterwards
                result.add_instruction(AsmOperation::SUB {decrement: Operand::Imm(increment_amount), data_type: original_type.clone() });

                //pop &self.operand to RCX
                result.add_instruction(AsmOperation::MOV {
                    to: RegOrMem::GPReg(GPRegister::secondary()),
                    from: Operand::Mem(MemoryOperand::SubFromBP(operand_address_storage)),
                    size: PTR_SIZE
                });

                //save the new value of self.operand
                result.add_instruction(AsmOperation::MOV {
                    to: RegOrMem::Mem(MemoryOperand::MemoryAddress { pointer_reg: GPRegister::secondary() }),
                    from: Operand::GPReg(GPRegister::acc()),
                    size: original_type.memory_size(asm_data)
                });

                let cast_asm = cast_from_acc(&original_type, &promoted_type, asm_data);//cast to the correct type
                result.merge(&cast_asm);

            },

            UnaryPrefixOperator::BooleanNot => {
                result.add_comment("boolean not");

                let original_type = self.operand.accept(&mut GetDataTypeVisitor {asm_data}).decay_to_primative();

                let operand_asm = self.operand.accept(&mut ScalarInAccVisitor {asm_data, stack_data});
                let cast_asm = cast_raw_from_acc(&original_type, &ScalarType::Integer(IntegerType::_BOOL), asm_data);//cast to boolean
                result.merge(&operand_asm);
                result.merge(&cast_asm);//cast to the correct type

                //compare the boolean to zero
                result.add_instruction(AsmOperation::CMP {
                    rhs: Operand::Imm(ImmediateValue("0".to_string())),
                    data_type: ScalarType::Integer(IntegerType::_BOOL),
                });

                //set 1 if equal to 0 or vice-versa
                result.add_instruction(AsmOperation::SETCC {
                    comparison: AsmComparison::EQ,//set to 1 if it was previously equal to 0
                });
            },

            UnaryPrefixOperator::BitwiseNot => {
                result.add_comment("boolean not");
                let promoted_type = self.get_data_type(asm_data);

                let original_type = self.operand.accept(&mut GetDataTypeVisitor {asm_data});

                let operand_asm = self.operand.accept(&mut ScalarInAccVisitor {asm_data, stack_data});
                let cast_asm = cast_from_acc(&original_type, &promoted_type, asm_data);//cast to boolean
                result.merge(&operand_asm);
                result.merge(&cast_asm);//cast to the correct type

                //set 1 if equal to 0 or vice-versa
                result.add_instruction(AsmOperation::BitwiseNot {
                    item: RegOrMem::GPReg(GPRegister::acc()),
                    size: promoted_type.memory_size(asm_data)
                });
            }
        }

        result
    }

    pub fn get_data_type(&self, asm_data: &AsmData) -> DataType {
        let operand_type = self.operand.accept(&mut GetDataTypeVisitor {asm_data});
        match self.operator {
            UnaryPrefixOperator::Reference => operand_type.add_outer_modifier(DeclModifier::POINTER),//pointer to whatever rhs is
            UnaryPrefixOperator::Dereference => operand_type.remove_outer_modifier(),
            UnaryPrefixOperator::UnaryPlus | UnaryPrefixOperator::Negate | UnaryPrefixOperator::Increment | UnaryPrefixOperator::Decrement | UnaryPrefixOperator::BitwiseNot => calculate_unary_type_arithmetic(&operand_type),//-x may promote x to a bigger type
            UnaryPrefixOperator::BooleanNot => DataType::RAW(BaseType::Scalar(ScalarType::Integer(IntegerType::_BOOL))),
        }
    }

    pub fn new(operator: UnaryPrefixOperator, operand: Expression) -> UnaryPrefixExpression {
        UnaryPrefixExpression { operand: Box::new(operand), operator }
    }

    pub fn get_operator(&self) -> &UnaryPrefixOperator {
        &self.operator
    }

    pub fn get_operand(&self) -> &Expression {
        &self.operand
    }
}

impl ASTDisplay for UnaryPrefixExpression {
    fn display_ast(&self, f: &mut crate::debugging::TreeDisplayInfo) {
        let operator_prefix: &str = self.operator.clone().into();
        f.write(&format!("prefix {}", operator_prefix.yellow()));

        f.indent();
        self.operand.display_ast(f);
        f.dedent();
    }
}