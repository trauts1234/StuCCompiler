
use crate::{asm_boilerplate::{self}, asm_gen_data::AsmData, asm_generation::{LogicalRegister, RegisterName, PTR_SIZE}, data_type::{data_type::DataType, type_modifier::DeclModifier}, expression::Expression, expression_visitors::{data_type_visitor::GetDataTypeVisitor, put_scalar_in_acc::ScalarInAccVisitor, reference_assembly_visitor::ReferenceVisitor}, lexer::punctuator::Punctuator};
use std::fmt::Write;
use crate::asm_generation::{asm_line, asm_comment};

#[derive(Clone)]
pub struct UnaryPrefixExpression {
    operand: Box<Expression>,
    operator: Punctuator,
}

impl UnaryPrefixExpression {
    pub fn generate_assembly(&self, asm_data: &AsmData) -> String {
        let mut result = String::new();

        match self.operator {
            Punctuator::AMPERSAND => {
                asm_comment!(result, "getting address of something");
                //put address of the right hand side in acc
                let operand_ref_asm = self.operand.accept(&mut ReferenceVisitor {asm_data});
                asm_line!(result, "{}", operand_ref_asm);
            },
            Punctuator::ASTERISK => {
                asm_comment!(result, "dereferencing pointer");
                // put the address pointed to in rax
                asm_line!(result, "{}", self.operand.accept(&mut ScalarInAccVisitor {asm_data}));
                if self.get_data_type(asm_data).is_array() {
                    //dereferencing results in an array, so I leave the address in RAX for future indexing etc.
                } else {
                    asm_line!(result, "mov rax, [rax]");//dereference pointer
                }
            },
            Punctuator::DASH => {
                asm_comment!(result, "negating something");

                let promoted_type = self.get_data_type(asm_data);

                asm_line!(result, "{}", self.operand.accept(&mut ScalarInAccVisitor {asm_data}));
                asm_line!(result, "{}", asm_boilerplate::cast_from_acc(&self.operand.accept(&mut GetDataTypeVisitor {asm_data}), &promoted_type));//cast to the correct type

                asm_line!(result, "neg {}", LogicalRegister::ACC.generate_reg_name(&promoted_type.memory_size()));//negate the promoted value
            },
            Punctuator::PLUSPLUS => {

                let promoted_type = self.get_data_type(asm_data);
                let rhs_type = self.operand.accept(&mut GetDataTypeVisitor {asm_data});

                //push &self.operand
                let operand_asm = self.operand.accept(&mut ReferenceVisitor {asm_data});
                asm_line!(result, "{}", operand_asm);
                asm_line!(result, "{}", asm_boilerplate::push_reg(&PTR_SIZE, &LogicalRegister::ACC));

                //put self.operand in acc
                asm_line!(result, "{}", self.operand.accept(&mut ScalarInAccVisitor {asm_data}));

                let rhs_reg = LogicalRegister::ACC.generate_reg_name(&rhs_type.memory_size());

                //increment self.operand (in acc)
                asm_line!(result, "inc {}", rhs_reg);

                //pop &self.operand to RCX
                asm_line!(result, "{}", asm_boilerplate::pop_reg(&PTR_SIZE, &LogicalRegister::SECONDARY));

                //save the new value of self.operand
                asm_line!(result, "mov [{}], {}", LogicalRegister::SECONDARY.generate_reg_name(&PTR_SIZE), LogicalRegister::ACC.generate_reg_name(&rhs_type.memory_size()));

                asm_line!(result, "{}", asm_boilerplate::cast_from_acc(&self.operand.accept(&mut GetDataTypeVisitor {asm_data}), &promoted_type));//cast to the correct type
            }
            _ => panic!("operator to unary prefix is invalid")
        }

        result
    }

    pub fn get_data_type(&self, asm_data: &AsmData) -> DataType {
        let operand_type = self.operand.accept(&mut GetDataTypeVisitor {asm_data});
        match self.operator {
            Punctuator::AMPERSAND => {
                let mut pointer_modifiers = operand_type.get_modifiers().to_vec();
                pointer_modifiers.insert(0, DeclModifier::POINTER);//pointer to whatever rhs is

                DataType::new_from_base_type(operand_type.underlying_type(), &pointer_modifiers)
            },
            Punctuator::ASTERISK => DataType::new_from_base_type(
                operand_type.underlying_type(), 
                &operand_type.get_modifiers()[1..].to_vec()//remove initial "pointer to x" from modifiers
            ),
            Punctuator::DASH | Punctuator::PLUSPLUS => DataType::calculate_unary_type_arithmetic(&operand_type),//-x may promote x to a bigger type
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