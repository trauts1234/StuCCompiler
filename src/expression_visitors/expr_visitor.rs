use crate::{binary_expression::BinaryExpression, cast_expr::CastExpression, declaration::MinimalDataVariable, expression::{sizeof_expression::SizeofExpr, ternary::TernaryExpr, unary_postfix_expression::UnaryPostfixExpression, unary_prefix_expr::UnaryPrefixExpression}, function_call::FunctionCall, number_literal::typed_value::NumberLiteral, string_literal::StringLiteral, member_access::MemberAccess};


//a test to see if a visitor pattern would be useful
pub trait ExprVisitor {
    type Output;

    fn visit_number_literal(&mut self, number: &NumberLiteral) -> Self::Output;
    fn visit_variable(&mut self, var: &MinimalDataVariable) -> Self::Output;
    fn visit_string_literal(&mut self, string: &StringLiteral) -> Self::Output;
    fn visit_func_call(&mut self, func_call: &FunctionCall) -> Self::Output;
    fn visit_unary_prefix(&mut self, expr: &UnaryPrefixExpression) -> Self::Output;
    fn visit_unary_postfix(&mut self, expr: &UnaryPostfixExpression) -> Self::Output;
    fn visit_binary_expression(&mut self, expr: &BinaryExpression) -> Self::Output;
    fn visit_member_access(&mut self, expr: &MemberAccess) -> Self::Output;
    fn visit_cast_expr(&mut self, expr: &CastExpression) -> Self::Output;
    fn visit_sizeof(&mut self, sizeof: &SizeofExpr) -> Self::Output;
    fn visit_ternary(&mut self, ternary: &TernaryExpr) -> Self::Output;
}