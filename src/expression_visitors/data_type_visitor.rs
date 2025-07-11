use crate::{asm_gen_data::AsmData, binary_expression::BinaryExpression, data_type::{base_type::{BaseType, IntegerType, ScalarType}, recursive_data_type::DataType, type_modifier::DeclModifier}, declaration::MinimalDataVariable, expression::unary_prefix_expr::UnaryPrefixExpression, expression_visitors::expr_visitor::ExprVisitor, function_call::FunctionCall, number_literal::typed_value::NumberLiteral, string_literal::StringLiteral, struct_member_access::StructMemberAccess};

pub struct GetDataTypeVisitor<'a>{
    pub(crate) asm_data: &'a AsmData
}

impl<'a> ExprVisitor for GetDataTypeVisitor<'a> {
    type Output = DataType;

    fn visit_number_literal(&mut self, number: &NumberLiteral) -> Self::Output {
        DataType::RAW(BaseType::Scalar(number.get_data_type()))
    }

    fn visit_variable(&mut self, var: &MinimalDataVariable) -> Self::Output {
        self.asm_data.get_variable(&var.name).data_type.clone()
    }

    fn visit_string_literal(&mut self, string: &StringLiteral) -> Self::Output {
        DataType::new(BaseType::Scalar(ScalarType::Integer(IntegerType::I8)))//8 bit integer
        .add_outer_modifier(DeclModifier::ARRAY(string.get_num_chars() as u64))//but replace modifiers to change it to an array of integers
    }

    fn visit_func_call(&mut self, func_call: &FunctionCall) -> Self::Output {
        func_call.get_callee_decl().return_type.clone()
    }

    fn visit_unary_prefix(&mut self, expr: &UnaryPrefixExpression) -> Self::Output {
        expr.get_data_type(self.asm_data)
    }

    fn visit_unary_postfix(&mut self, expr: &crate::expression::unary_postfix_expression::UnaryPostfixExpression) -> Self::Output {
        expr.get_data_type(self.asm_data)
    }

    fn visit_binary_expression(&mut self, expr: &BinaryExpression) -> Self::Output {
        expr.get_data_type(self.asm_data)
    }

    fn visit_struct_member_access(&mut self, expr: &StructMemberAccess) -> Self::Output {
        expr.get_data_type(self.asm_data)
    }
    
    fn visit_cast_expr(&mut self, expr: &crate::cast_expr::CastExpression) -> Self::Output {
        expr.get_new_type().clone()
    }
    
    fn visit_sizeof(&mut self, _: &crate::expression::sizeof_expression::SizeofExpr) -> Self::Output {
        DataType::RAW(BaseType::Scalar(ScalarType::Integer(IntegerType::U64)))//sizeof is size_t-sized
    }
}