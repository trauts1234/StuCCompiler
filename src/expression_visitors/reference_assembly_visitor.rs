use stack_management::simple_stack_frame::SimpleStackFrame;

use crate::{asm_gen_data::{AsmData, GlobalAsmData}, assembly::{assembly::Assembly, operand::memory_operand::MemoryOperand, operation::AsmOperation}, expression::{unary_prefix_expr::UnaryPrefixExpression, unary_prefix_operator::UnaryPrefixOperator}, expression_visitors::{expr_visitor::ExprVisitor}, member_access::MemberAccess};

/**
 * puts the address of the visited Expression in RAX
 */
pub struct ReferenceVisitor<'a>{
    pub(crate) asm_data: &'a AsmData,
    pub(crate) stack_data: &'a mut SimpleStackFrame,
    pub(crate) global_asm_data: &'a mut GlobalAsmData
}

impl<'a> ExprVisitor for ReferenceVisitor<'a> {
    type Output = Assembly;

    fn visit_number_literal(&mut self, _number: &crate::number_literal::typed_value::NumberLiteral) -> Self::Output {
        panic!("tried to get address of number literal")
    }

    fn visit_variable(&mut self, var: &crate::declaration::MinimalDataVariable) -> Self::Output {

        let mut result = Assembly::make_empty();

        result.add_instruction(AsmOperation::LEA {
            from: self.asm_data.get_variable(&var.name).location.clone(),
        });

        result
    }

    fn visit_string_literal(&mut self, string: &crate::string_literal::StringLiteral) -> Self::Output {
        let mut result = Assembly::make_empty();

        result.add_instruction(AsmOperation::LEA {
            from: MemoryOperand::LabelAccess(string.get_label().to_string()),
        });

        result
    }

    fn visit_func_call(&mut self, _func_call: &crate::function_call::FunctionCall) -> Self::Output {
        panic!("cannot get address of function call")
    }

    fn visit_unary_prefix(&mut self, expr: &UnaryPrefixExpression) -> Self::Output {
        let mut result = Assembly::make_empty();
        //&*x == x
        assert!(*expr.get_operator() == UnaryPrefixOperator::Dereference);//must be address of a dereference

        let operand_asm = expr.get_operand().accept(&mut ScalarInAccVisitor{asm_data: self.asm_data, stack_data: self.stack_data, global_asm_data: self.global_asm_data});

        result.add_comment("getting address of a dereference");
        result.merge(&operand_asm);

        result
    }

    fn visit_unary_postfix(&mut self, _expr: &crate::expression::unary_postfix_expression::UnaryPostfixExpression) -> Self::Output {
        panic!("tried to get address of unary postfix")
    }

    fn visit_binary_expression(&mut self, _expr: &crate::binary_expression::BinaryExpression) -> Self::Output {
        panic!("tried to get address of binary expression")
    }

    fn visit_member_access(&mut self, member_access: &MemberAccess) -> Self::Output {
        member_access.put_addr_in_acc(self.asm_data, self.stack_data, self.global_asm_data)
    }
    
    fn visit_cast_expr(&mut self, _: &crate::cast_expr::CastExpression) -> Self::Output {
        panic!("cannot get address of a cast")
    }
    
    fn visit_sizeof(&mut self, _: &crate::expression::sizeof_expression::SizeofExpr) -> Self::Output {
        panic!("cannot get address of a sizeof");
    }
    
    fn visit_ternary(&mut self, _: &crate::expression::ternary::TernaryExpr) -> Self::Output {
        panic!("cannot get address of a ternary operator")
    }
}