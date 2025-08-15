use crate::{asm_gen_data::AsmData, binary_expression::BinaryExpression, data_type::{base_type::{BaseType, IntegerType, ScalarType}, recursive_data_type::{calculate_promoted_type_arithmetic, calculate_unary_type_arithmetic, DataType}, type_modifier::DeclModifier}, declaration::MinimalDataVariable, expression::{binary_expression_operator::BinaryExpressionOperator, sizeof_expression::SizeofExpr, unary_prefix_expr::UnaryPrefixExpression}, expression_visitors::expr_visitor::ExprVisitor, function_call::FunctionCall, number_literal::typed_value::NumberLiteral, string_literal::StringLiteral, struct_member_access::StructMemberAccess};

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
        match expr.operator() {
            BinaryExpressionOperator::BitwiseOr |
            BinaryExpressionOperator::BitwiseAnd |
            BinaryExpressionOperator::BitwiseXor |
            BinaryExpressionOperator::Add |
            BinaryExpressionOperator::Subtract |
            BinaryExpressionOperator::Multiply | 
            BinaryExpressionOperator::Divide | 
            BinaryExpressionOperator::Mod => {
                calculate_promoted_type_arithmetic(//calculate type when data types:
                    &expr.lhs().accept(&mut GetDataTypeVisitor { asm_data: self.asm_data }),//type of lhs
                    &expr.rhs().accept(&mut GetDataTypeVisitor { asm_data: self.asm_data }),//type of rhs
                )
            },

            BinaryExpressionOperator::Assign |
            BinaryExpressionOperator::AdditionCombination |
            BinaryExpressionOperator::SubtractionCombination => expr.lhs().accept(&mut GetDataTypeVisitor {asm_data: self.asm_data}),//assigning, rhs must be converted to lhs

            //bit shifts have lhs promoted, then resultant type is the same as promoted lhs
            BinaryExpressionOperator::BitshiftLeft |
            BinaryExpressionOperator::BitshiftRight => calculate_unary_type_arithmetic(&expr.lhs().accept(&mut GetDataTypeVisitor {asm_data: self.asm_data})),

            BinaryExpressionOperator::CmpLess |
            BinaryExpressionOperator::CmpGreater |
            BinaryExpressionOperator::CmpGreaterEqual |
            BinaryExpressionOperator::CmpLessEqual |
            BinaryExpressionOperator::CmpEqual |
            BinaryExpressionOperator::CmpNotEqual |
            BinaryExpressionOperator::BooleanOr |
            BinaryExpressionOperator::BooleanAnd  => DataType::RAW(BaseType::Scalar(ScalarType::Integer(IntegerType::_BOOL))),
        }
    }

    fn visit_struct_member_access(&mut self, expr: &StructMemberAccess) -> Self::Output {
        expr.get_data_type(self.asm_data)
    }
    
    fn visit_cast_expr(&mut self, expr: &crate::cast_expr::CastExpression) -> Self::Output {
        expr.get_new_type().clone()
    }
    
    fn visit_sizeof(&mut self, _: &SizeofExpr) -> Self::Output {
        DataType::RAW(BaseType::Scalar(ScalarType::Integer(IntegerType::U64)))//sizeof is size_t-sized
    }
    
    fn visit_ternary(&mut self, ternary: &crate::expression::ternary::TernaryExpr) -> Self::Output {
        calculate_promoted_type_arithmetic(
            &ternary.true_branch().accept(&mut GetDataTypeVisitor { asm_data: self.asm_data }),
            &ternary.false_branch().accept(&mut GetDataTypeVisitor { asm_data: self.asm_data })
        )
    }
}