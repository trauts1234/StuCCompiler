use crate::{asm_boilerplate::cast_from_acc, asm_gen_data::{AsmData, GetStructUnion, GlobalAsmData}, assembly::{assembly::Assembly, comparison::AsmComparison, operand::{immediate::{ImmediateValue, ToImmediate}, memory_operand::MemoryOperand, register::GPRegister, Operand, RegOrMem}, operation::{AsmOperation, Label}}, data_type::{base_type::{BaseType, IntegerType, ScalarType}, recursive_data_type::DataType}, expression::unary_prefix_expr::UnaryPrefixExpression, expression_visitors::{reference_assembly_visitor::ReferenceVisitor}, member_access::MemberAccess};
use stack_management::simple_stack_frame::SimpleStackFrame;
use unwrap_let::unwrap_let;
use uuid::Uuid;
use super::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor};


/**
 * calculates the value of the expression and puts the scalar result in AX
 * does not work with structs, as they are not scalar types
 * stack_data is modified
 */
pub struct ScalarInAccVisitor<'a>{
    pub(crate) asm_data: &'a AsmData,
    pub(crate) stack_data: &'a mut SimpleStackFrame,
    pub(crate) global_asm_data: &'a GlobalAsmData
}

impl<'a> ExprVisitor for ScalarInAccVisitor<'a> {
    type Output = Assembly;

    fn visit_number_literal(&mut self, number: &crate::number_literal::typed_value::NumberLiteral) -> Self::Output {
        let mut result = Assembly::make_empty();

        let reg_size = number.get_data_type().memory_size();//decide how much storage is needed to temporarily store the constant
        result.add_comment(format!("reading number literal: {:?}", number));

        result.add_instruction(AsmOperation::MOV {
            to: RegOrMem::GPReg(GPRegister::acc()),
            from: Operand::Imm(number.as_imm()),
            size: reg_size,
        });

        result
    }

    fn visit_variable(&mut self, var: &crate::declaration::MinimalDataVariable) -> Self::Output {
        let mut result = Assembly::make_empty();

        let my_type = var.accept(&mut GetDataTypeVisitor{asm_data: self.asm_data});

        if let DataType::ARRAY {..} = my_type {//is array
            //getting an array, decays to a pointer
            result.add_comment(format!("decaying array {} to pointer", var.name));
            let addr_asm = var.accept(&mut ReferenceVisitor{asm_data: self.asm_data, stack_data: self.stack_data, global_asm_data: self.global_asm_data});
            result.merge(&addr_asm);

        } else {
            let reg_size = my_type.memory_size(self.asm_data);//decide which register size is appropriate for this variable
            result.add_comment(format!("reading variable: {}", var.name));

            result.add_instruction(AsmOperation::MOV {
                to: RegOrMem::GPReg(GPRegister::acc()),
                from: Operand::Mem(self.asm_data.get_variable(&var.name).location.clone()),
                size: reg_size,
            });
        }

        result
    }

    fn visit_string_literal(&mut self, string: &crate::string_literal::StringLiteral) -> Self::Output {
        let mut result = Assembly::make_empty();
        result.add_instruction(AsmOperation::LEA {
            from: MemoryOperand::LabelAccess(string.get_label().to_string())
        });//warning: duplicated code from get address

        result
    }

    fn visit_func_call(&mut self, func_call: &crate::function_call::FunctionCall) -> Self::Output {
        panic!("put it on the stack instead");
    }

    fn visit_unary_prefix(&mut self, expr: &UnaryPrefixExpression) -> Self::Output {
        expr.generate_assembly(self.asm_data, self.stack_data, self.global_asm_data)
    }

    fn visit_unary_postfix(&mut self, expr: &crate::expression::unary_postfix_expression::UnaryPostfixExpression) -> Self::Output {
        expr.generate_assembly(self.asm_data, self.stack_data, self.global_asm_data)
    }

    fn visit_binary_expression(&mut self, expr: &crate::binary_expression::BinaryExpression) -> Self::Output {
        expr.generate_assembly(self.asm_data, self.stack_data, self.global_asm_data)
    }

    fn visit_member_access(&mut self, member_access: &MemberAccess) -> Self::Output {
        panic!()
    }
    
    fn visit_cast_expr(&mut self, expr: &crate::cast_expr::CastExpression) -> Self::Output {
        let mut result = Assembly::make_empty();
        let uncasted_asm = expr.get_uncasted_expr().accept(self);
        let uncasted_type = expr.get_uncasted_expr().accept(&mut GetDataTypeVisitor{asm_data:self.asm_data});
        //generate uncasted data
        result.merge(&uncasted_asm);
        //cast to required type
        result.merge(&cast_from_acc(&uncasted_type, expr.get_new_type(), self.asm_data));

        result
    }
    
    fn visit_sizeof(&mut self, sizeof: &crate::expression::sizeof_expression::SizeofExpr) -> Self::Output {
        sizeof.generate_assembly(self.asm_data)
    }
    
    fn visit_ternary(&mut self, ternary: &crate::expression::ternary::TernaryExpr) -> Self::Output {
        let mut result = Assembly::make_empty();

        let generic_label = Uuid::new_v4().simple().to_string();
        let else_label = Label::Local(format!("{}_else", generic_label));//jump for the else branch
        let if_end_label = Label::Local(format!("{}_end", generic_label));//rendevous point for the if and else branches

        let cond_false_label = &else_label;//only jump to else branch if it exists

        let condition_asm = ternary.condition().accept(&mut ScalarInAccVisitor {asm_data: self.asm_data, stack_data: self.stack_data, global_asm_data: self.global_asm_data});
        result.merge(&condition_asm);//generate the condition to acc
        
        unwrap_let!(DataType::RAW(BaseType::Scalar(condition_type)) = ternary.condition().accept(&mut GetDataTypeVisitor {asm_data: self.asm_data}));

        //compare the result to 0
        result.add_instruction(AsmOperation::CMP {
            rhs: Operand::Imm(ImmediateValue("0".to_string())),
            data_type: condition_type
        });

        //if the result is 0, jump to the else block or the end of the if statement
        result.add_instruction(AsmOperation::JMPCC {
            label: cond_false_label.clone(),
            comparison: AsmComparison::EQ,
        });

        //generate the body of the if statement
        let if_body_asm = ternary.true_branch().accept(&mut ScalarInAccVisitor {asm_data: self.asm_data, stack_data: &mut self.stack_data, global_asm_data: self.global_asm_data});
        result.merge(&if_body_asm);

        //jump to the end of the if/else block
        result.add_instruction(AsmOperation::JMPCC {
            label: if_end_label.clone(),
            comparison: AsmComparison::ALWAYS,//unconditional jump
        });

        let else_body_asm = ternary.false_branch().accept(&mut ScalarInAccVisitor {asm_data: self.asm_data, stack_data: &mut self.stack_data, global_asm_data: self.global_asm_data});

        //start of the else block
        result.add_instruction(AsmOperation::Label(else_label));//add label
        result.merge(&else_body_asm);//generate the body of the else statement

        //after if/else are complete, jump here
        result.add_instruction(AsmOperation::Label(if_end_label));

        result

    }
}