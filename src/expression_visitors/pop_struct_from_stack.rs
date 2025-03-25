use crate::{asm_gen_data::AsmData, asm_generation::asm_line, data_type::data_type::DataType, expression_visitors::data_type_visitor::GetDataTypeVisitor};
use std::fmt::Write;
use unwrap_let::unwrap_let;
use super::expr_visitor::ExprVisitor;


pub struct PopStructFromStack<'a>{
    pub(crate) asm_data: &'a AsmData
}

impl<'a> ExprVisitor for PopStructFromStack<'a> {
    type Output = String;

    fn visit_number_literal(&mut self, _number: &crate::number_literal::NumberLiteral) -> Self::Output {
        panic!()
    }

    fn visit_variable(&mut self, _: &crate::declaration::MinimalDataVariable) -> Self::Output {
        String::new()//accessing stack variable does not allocate so needs no deallocation
    }

    fn visit_string_literal(&mut self, _string: &crate::string_literal::StringLiteral) -> Self::Output {
        panic!();
    }

    fn visit_func_call(&mut self, func_call: &crate::function_call::FunctionCall) -> Self::Output {
        let mut result = String::new();

        unwrap_let!(DataType::COMPOSITE(return_value) = func_call.accept(&mut GetDataTypeVisitor{asm_data: self.asm_data}));

        let callee_name = &func_call.get_callee_decl().function_name;

        asm_line!(result, "add rsp, {} ; deallocate a struct returned from a function call to {}", return_value.memory_size(self.asm_data).size_bytes(), callee_name);

        result
    }

    fn visit_unary_prefix(&mut self, _: &crate::unary_prefix_expr::UnaryPrefixExpression) -> Self::Output {
        String::new()//pointer dereference does not allocate, so no deallocation needed
    }

    fn visit_binary_expression(&mut self, _expr: &crate::binary_expression::BinaryExpression) -> Self::Output {
        panic!();
    }

    fn visit_struct_member_access(&mut self, expr: &crate::struct_definition::StructMemberAccess) -> Self::Output {
        expr.get_base_struct_tree().accept(&mut PopStructFromStack{asm_data:self.asm_data})//pop base struct that I got my struct from, as that may allocate: foo().x_member
    }
}