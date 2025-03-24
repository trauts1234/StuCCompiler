use crate::{binary_expression::BinaryExpression, declaration::MinimalDataVariable, function_call::FunctionCall, number_literal::NumberLiteral, string_literal::StringLiteral, struct_definition::StructMemberAccess, unary_prefix_expr::UnaryPrefixExpression};


//a test to see if a visitor pattern would be useful
pub trait ExprVisitor {
    type Output;

    fn visit_number_literal(&mut self, number: &NumberLiteral) -> Self::Output;
    fn visit_variable(&mut self, var: &MinimalDataVariable) -> Self::Output;
    fn visit_string_literal(&mut self, string: &StringLiteral) -> Self::Output;
    fn visit_func_call(&mut self, func_call: &FunctionCall) -> Self::Output;
    fn visit_unary_prefix(&mut self, expr: &UnaryPrefixExpression) -> Self::Output;
    fn visit_binary_expression(&mut self, expr: &BinaryExpression) -> Self::Output;
    fn visit_struct_member_access(&mut self, expr: &StructMemberAccess) -> Self::Output;

}