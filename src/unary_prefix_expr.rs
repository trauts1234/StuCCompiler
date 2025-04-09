use crate::{asm_boilerplate::cast_from_acc, asm_gen_data::AsmData, assembly::{assembly::Assembly, operand::{immediate::ImmediateValue, memory_operand::MemoryOperand, register::Register, Operand, RegOrMem, PTR_SIZE}, operation::AsmOperation}, data_type::{recursive_data_type::{calculate_unary_type_arithmetic, DataType}, type_modifier::DeclModifier}, expression::Expression, expression_visitors::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor, put_scalar_in_acc::ScalarInAccVisitor, reference_assembly_visitor::ReferenceVisitor}, lexer::punctuator::Punctuator, memory_size::MemoryLayout};

#[derive(Clone)]
pub struct UnaryPrefixExpression {
    operand: Box<Expression>,
    operator: Punctuator,
}

impl UnaryPrefixExpression {
    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_unary_prefix(self)
    }
    
    pub fn generate_assembly(&self, asm_data: &AsmData, stack_data: &mut MemoryLayout) -> Assembly {
        let mut result = Assembly::make_empty();

        match self.operator {
            Punctuator::AMPERSAND => {
                result.add_comment("getting address of something");
                //put address of the right hand side in acc
                let operand_ref_asm = self.operand.accept(&mut ReferenceVisitor {asm_data, stack_data});
                result.merge(&operand_ref_asm);
            },
            Punctuator::ASTERISK => {
                result.add_comment("dereferencing pointer");
                // put the address pointed to in rax
                let operand_asm = self.operand.accept(&mut ScalarInAccVisitor {asm_data, stack_data});
                result.merge(&operand_asm);

                if let DataType::ARRAY {..} = self.get_data_type(asm_data) {
                    //dereferencing results in an array, so I leave the address in RAX for future indexing etc.
                } else {
                    result.add_instruction(AsmOperation::MOV {
                        to: RegOrMem::Reg(Register::acc()),
                        from: Operand::Mem(MemoryOperand::MemoryAddress { pointer_reg: Register::acc() }),
                        size: PTR_SIZE
                    });//dereference pointer
                }
            },
            Punctuator::DASH => {
                result.add_comment("negating something");

                let promoted_type = self.get_data_type(asm_data);
                let original_type = self.operand.accept(&mut GetDataTypeVisitor {asm_data});

                let operand_asm = self.operand.accept(&mut ScalarInAccVisitor {asm_data, stack_data});
                let cast_asm = cast_from_acc(&original_type, &promoted_type, asm_data);
                result.merge(&operand_asm);
                result.merge(&cast_asm);//cast to the correct type

                result.add_instruction(AsmOperation::NEG { item: RegOrMem::Reg(Register::acc()), data_type: promoted_type });//negate the promoted value
            },
            Punctuator::PLUSPLUS => {

                let promoted_type = self.get_data_type(asm_data);
                let original_type = self.operand.accept(&mut GetDataTypeVisitor {asm_data});

                //push &self.operand
                let operand_asm = self.operand.accept(&mut ReferenceVisitor {asm_data, stack_data});
                result.merge(&operand_asm);
                *stack_data += PTR_SIZE;//allocate temporary lhs storage
                let operand_address_storage = stack_data.clone();
                result.add_instruction(AsmOperation::MOV {
                    to: RegOrMem::Mem(MemoryOperand::SubFromBP(operand_address_storage)),
                    from: Operand::Reg(Register::acc()),
                    size: PTR_SIZE
                });

                //put self.operand in acc
                let operand_asm = self.operand.accept(&mut ScalarInAccVisitor {asm_data, stack_data});
                result.merge(&operand_asm);

                let rhs_reg = RegOrMem::Reg(Register::acc());
                //increment self.operand (in acc) as original type, so that it can be stored correctly afterwards
                result.add_instruction(AsmOperation::ADD { destination: rhs_reg, increment: Operand::Imm(ImmediateValue("1".to_string())), data_type: original_type.clone() });

                //pop &self.operand to RCX
                result.add_instruction(AsmOperation::MOV {
                    to: RegOrMem::Reg(Register::secondary()),
                    from: Operand::Mem(MemoryOperand::SubFromBP(operand_address_storage)),
                    size: PTR_SIZE
                });

                //save the new value of self.operand
                result.add_instruction(AsmOperation::MOV {
                    to: RegOrMem::Mem(MemoryOperand::MemoryAddress { pointer_reg: Register::secondary() }),
                    from: Operand::Reg(Register::acc()),
                    size: original_type.memory_size(asm_data)
                });

                let cast_asm = cast_from_acc(&original_type, &promoted_type, asm_data);//cast to the correct type
                result.merge(&cast_asm);

            }
            _ => panic!("operator to unary prefix is invalid")
        }

        result
    }

    pub fn get_data_type(&self, asm_data: &AsmData) -> DataType {
        let operand_type = self.operand.accept(&mut GetDataTypeVisitor {asm_data});
        match self.operator {
            Punctuator::AMPERSAND => operand_type.add_outer_modifier(DeclModifier::POINTER),//pointer to whatever rhs is
            Punctuator::ASTERISK => operand_type.remove_outer_modifier(),
            Punctuator::DASH | Punctuator::PLUSPLUS => calculate_unary_type_arithmetic(&operand_type, asm_data),//-x may promote x to a bigger type
            _ => panic!("tried getting data type of a not-implemented prefix")
        }
    }
}

impl UnaryPrefixExpression {
    pub fn new(operator: Punctuator, operand: Expression) -> UnaryPrefixExpression {
        UnaryPrefixExpression { operand: Box::new(operand), operator }
    }

    pub fn get_operator(&self) -> &Punctuator {
        &self.operator
    }

    pub fn get_operand(&self) -> &Expression {
        &self.operand
    }
}