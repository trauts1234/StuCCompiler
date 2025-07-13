use colored::Colorize;
use memory_size::MemorySize;

use crate::{asm_boilerplate::cast_from_acc, asm_gen_data::AsmData, assembly::{assembly::Assembly, operand::{immediate::{ImmediateValue, MemorySizeExt}, memory_operand::MemoryOperand, register::GPRegister, Operand, RegOrMem, PTR_SIZE}, operation::AsmOperation}, data_type::{base_type::IntegerType, recursive_data_type::{calculate_unary_type_arithmetic, DataType}}, debugging::ASTDisplay, expression_visitors::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor, reference_assembly_visitor::ReferenceVisitor}, number_literal::typed_value::NumberLiteral};

use super::{expression::Expression, unary_postfix_operator::UnaryPostfixOperator};

#[derive(Clone, Debug)]
pub struct UnaryPostfixExpression {
    operand: Box<Expression>,
    operator: UnaryPostfixOperator,
}

impl UnaryPostfixExpression {
    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_unary_postfix(self)
    }

    pub fn get_data_type(&self, asm_data: &AsmData) -> DataType {
        let operand_type = self.operand.accept(&mut GetDataTypeVisitor {asm_data});
        match self.operator {
            UnaryPostfixOperator::Increment |
            UnaryPostfixOperator::Decrement => calculate_unary_type_arithmetic(&operand_type),//-x may promote x to a bigger type
        }
    }

    pub fn generate_assembly(&self, asm_data: &AsmData, stack_data: &mut MemorySize) -> Assembly {
        let mut result = Assembly::make_empty();

        match self.operator {
            UnaryPostfixOperator::Decrement => {
                result.add_comment("postfix decrement");
                let promoted_type = self.get_data_type(asm_data);
                let original_type = self.operand.accept(&mut GetDataTypeVisitor {asm_data});

                let increment_amount = match &original_type {
                    DataType::UNKNOWNSIZEARRAY { .. } |
                    DataType::ARRAY {..} => panic!("this operation is invalid for arrays"),
                    
                    DataType::POINTER(underlying) => underlying.memory_size(asm_data).as_imm(),//increment pointer adds number of bytes
                    DataType::RAW(_) => ImmediateValue("1".to_string())
                };

                //put address of operand in secondary
                let operand_asm = self.operand.accept(&mut ReferenceVisitor {asm_data, stack_data});
                result.merge(&operand_asm);
                result.add_instruction(AsmOperation::MOV {
                    to: RegOrMem::GPReg(GPRegister::secondary()),
                    from: Operand::GPReg(GPRegister::acc()),
                    size: PTR_SIZE
                });

                //put self.operand in acc and third
                result.add_instruction(AsmOperation::MOV {
                    to: RegOrMem::GPReg(GPRegister::acc()),
                    from: Operand::Mem(MemoryOperand::MemoryAddress { pointer_reg: GPRegister::secondary() }),
                    size: original_type.memory_size(asm_data),
                });
                result.add_instruction(AsmOperation::MOV {
                    to: RegOrMem::GPReg(GPRegister::third()),
                    from: Operand::GPReg(GPRegister::acc()),
                    size: original_type.memory_size(asm_data),
                });

                //increment value in acc
                result.add_instruction(AsmOperation::SUB {
                    decrement: Operand::Imm(increment_amount),
                    data_type: original_type.clone(),
                });
                //save acc to the variable address
                result.add_instruction(AsmOperation::MOV {
                    to: RegOrMem::Mem(MemoryOperand::MemoryAddress { pointer_reg: GPRegister::secondary() }),
                    from: Operand::GPReg(GPRegister::acc()),
                    size: original_type.memory_size(asm_data),
                });

                //resurrect the old value
                result.add_instruction(AsmOperation::MOV {
                    to: RegOrMem::GPReg(GPRegister::acc()),
                    from: Operand::GPReg(GPRegister::third()),
                    size: original_type.memory_size(asm_data),
                });

                //promote the original self.operand in acc
                let cast_asm = cast_from_acc(&original_type, &promoted_type, asm_data);
                result.merge(&cast_asm);
            },
            UnaryPostfixOperator::Increment => {
                result.add_comment("postfix increment");
                let promoted_type = self.get_data_type(asm_data);
                let original_type = self.operand.accept(&mut GetDataTypeVisitor {asm_data});

                let increment_amount = match &original_type {
                    DataType::UNKNOWNSIZEARRAY { .. } |
                    DataType::ARRAY {..} => panic!("this operation is invalid for arrays"),

                    DataType::POINTER(underlying) => underlying.memory_size(asm_data).as_imm(),//increment pointer adds number of bytes
                    DataType::RAW(_) => ImmediateValue("1".to_string())
                };

                //put address of operand in secondary
                let operand_asm = self.operand.accept(&mut ReferenceVisitor {asm_data, stack_data});
                result.merge(&operand_asm);
                result.add_instruction(AsmOperation::MOV {
                    to: RegOrMem::GPReg(GPRegister::secondary()),
                    from: Operand::GPReg(GPRegister::acc()),
                    size: PTR_SIZE
                });

                //put self.operand in acc and third
                result.add_instruction(AsmOperation::MOV {
                    to: RegOrMem::GPReg(GPRegister::acc()),
                    from: Operand::Mem(MemoryOperand::MemoryAddress { pointer_reg: GPRegister::secondary() }),
                    size: original_type.memory_size(asm_data),
                });
                result.add_instruction(AsmOperation::MOV {
                    to: RegOrMem::GPReg(GPRegister::third()),
                    from: Operand::GPReg(GPRegister::acc()),
                    size: original_type.memory_size(asm_data),
                });

                //increment value in acc
                result.add_instruction(AsmOperation::ADD {
                    increment: Operand::Imm(increment_amount),
                    data_type: original_type.clone(),
                });
                //save acc to the variable address
                result.add_instruction(AsmOperation::MOV {
                    to: RegOrMem::Mem(MemoryOperand::MemoryAddress { pointer_reg: GPRegister::secondary() }),
                    from: Operand::GPReg(GPRegister::acc()),
                    size: original_type.memory_size(asm_data),
                });

                //promote the original self.operand
                result.add_instruction(AsmOperation::MOV {
                    to: RegOrMem::GPReg(GPRegister::acc()),
                    from: Operand::GPReg(GPRegister::third()),
                    size: original_type.memory_size(asm_data),
                });
                let cast_asm = cast_from_acc(&original_type, &promoted_type, asm_data);
                result.merge(&cast_asm);

            },
        }

        result
    }

    pub fn new(operator: UnaryPostfixOperator, operand: Expression) -> Self{
        Self { operand: Box::new(operand), operator }
    }

    pub fn get_operator(&self) -> &UnaryPostfixOperator {
        &self.operator
    }

    pub fn get_operand(&self) -> &Expression {
        &self.operand
    }
}

impl ASTDisplay for UnaryPostfixExpression {
    fn display_ast(&self, f: &mut crate::debugging::TreeDisplayInfo) {
        let operator_prefix: &str = self.operator.clone().into();
        f.write(&format!("postfix {}", operator_prefix.yellow()));

        f.indent();
        self.operand.display_ast(f);
        f.dedent();
    }
}