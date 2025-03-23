use crate::{asm_gen_data::AsmData, binary_expression::BinaryExpression, data_type::{base_type::BaseType, data_type::DataType, type_modifier::DeclModifier}, declaration::MinimalDataVariable, expression::ExprVisitor, function_call::FunctionCall, number_literal::NumberLiteral, string_literal::StringLiteral, struct_definition::StructMemberAccess, unary_prefix_expr::UnaryPrefixExpression};

pub struct GetDataTypeVisitor;

impl ExprVisitor for GetDataTypeVisitor {
    type Output = DataType;

    fn visit_number_literal(&mut self, number: &NumberLiteral) -> Self::Output {
        number.get_data_type()
    }

    fn visit_variable(&mut self, var: &MinimalDataVariable, asm_data: &AsmData) -> Self::Output {
        asm_data.get_variable(&var.name).data_type.clone()
    }

    fn visit_string_literal(&mut self, string: &StringLiteral) -> Self::Output {
        DataType::new_from_base_type(&BaseType::I8, &vec![DeclModifier::ARRAY(string.get_num_chars())])
    }

    fn visit_func_call(&mut self, func_call: &FunctionCall, _asm_data: &AsmData) -> Self::Output {
        func_call.get_callee_decl().return_type.clone()
    }

    fn visit_unary_prefix(&mut self, expr: &UnaryPrefixExpression, asm_data: &AsmData) -> Self::Output {
        expr.get_data_type(asm_data)
    }

    fn visit_binary_expression(&mut self, expr: &BinaryExpression, asm_data: &AsmData) -> Self::Output {
        expr.get_data_type(asm_data)
    }

    fn visit_struct_member_access(&mut self, expr: &StructMemberAccess, asm_data: &AsmData) -> Self::Output {
        expr.get_data_type(asm_data)
    }
}