use crate::{asm_boilerplate::{self}, asm_gen_data::AsmData, asm_generation::{LogicalRegister, RegisterName, PTR_SIZE}, data_type::{recursive_data_type::{calculate_unary_type_arithmetic, RecursiveDataType}, type_modifier::DeclModifier}, expression::Expression, expression_visitors::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor, put_scalar_in_acc::ScalarInAccVisitor, reference_assembly_visitor::ReferenceVisitor}, lexer::punctuator::Punctuator};
use std::fmt::Write;
use crate::asm_generation::{asm_line, asm_comment};

#[derive(Clone)]
pub struct UnaryPrefixExpression {
    operand: Box<Expression>,
    operator: Punctuator,
}

impl UnaryPrefixExpression {
    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_unary_prefix(self)
    }
    
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
                if let RecursiveDataType::ARRAY {..} = self.get_data_type(asm_data) {
                    //dereferencing results in an array, so I leave the address in RAX for future indexing etc.
                } else {
                    asm_line!(result, "mov rax, [rax]");//dereference pointer
                }
            },
            Punctuator::DASH => {
                asm_comment!(result, "negating something");

                let promoted_type = self.get_data_type(asm_data);
                let original_type = self.operand.accept(&mut GetDataTypeVisitor {asm_data});

                asm_line!(result, "{}", self.operand.accept(&mut ScalarInAccVisitor {asm_data}));
                asm_line!(result, "{}", asm_boilerplate::cast_from_acc(&original_type, &promoted_type, asm_data));//cast to the correct type

                asm_line!(result, "neg {}", LogicalRegister::ACC.generate_reg_name(&promoted_type.memory_size(asm_data)));//negate the promoted value
            },
            Punctuator::PLUSPLUS => {

                let promoted_type = self.get_data_type(asm_data);
                let original_type = self.operand.accept(&mut GetDataTypeVisitor {asm_data});

                //push &self.operand
                let operand_asm = self.operand.accept(&mut ReferenceVisitor {asm_data});
                asm_line!(result, "{}", operand_asm);
                asm_line!(result, "{}", asm_boilerplate::push_reg(&PTR_SIZE, &LogicalRegister::ACC));

                //put self.operand in acc
                asm_line!(result, "{}", self.operand.accept(&mut ScalarInAccVisitor {asm_data}));

                let rhs_reg = LogicalRegister::ACC.generate_reg_name(&original_type.memory_size(asm_data));

                //increment self.operand (in acc)
                asm_line!(result, "inc {}", rhs_reg);

                //pop &self.operand to RCX
                asm_line!(result, "{}", asm_boilerplate::pop_reg(&PTR_SIZE, &LogicalRegister::SECONDARY));

                //save the new value of self.operand
                asm_line!(result, "mov [{}], {}", LogicalRegister::SECONDARY.generate_reg_name(&PTR_SIZE), LogicalRegister::ACC.generate_reg_name(&original_type.memory_size(asm_data)));

                asm_line!(result, "{}", asm_boilerplate::cast_from_acc(&original_type, &promoted_type, asm_data));//cast to the correct type
            }
            _ => panic!("operator to unary prefix is invalid")
        }

        result
    }

    pub fn get_data_type(&self, asm_data: &AsmData) -> RecursiveDataType {
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