use crate::{asm_gen_data::{AsmData, VariableAddress}, asm_generation::asm_comment};
use std::fmt::Write;
use super::expr_visitor::ExprVisitor;



pub struct PutStructOnStack<'a>{
    pub(crate) asm_data: &'a AsmData
}

    //note: this does not allocate if expression is a variable

impl<'a> ExprVisitor for PutStructOnStack<'a> {
    type Output = String;

    fn visit_number_literal(&mut self, number: &crate::number_literal::NumberLiteral) -> Self::Output {
        panic!("tried to put struct on stack but found number literal")
    }

    fn visit_variable(&mut self, var: &crate::declaration::MinimalDataVariable) -> Self::Output {
        todo!("reduce code duplication from get address to acc?");
        let mut result = String::new();

        asm_comment!(result, "getting address of struct {} instead of copying to stack", var.name);

        match &self.asm_data.get_variable(&var.name).location {
            VariableAddress::CONSTANTADDRESS => 
                todo!(),
            VariableAddress::STACKOFFSET(stack_offset) => 
                todo!()
        }

        result
    }

    fn visit_string_literal(&mut self, string: &crate::string_literal::StringLiteral) -> Self::Output {
        todo!()
    }

    fn visit_func_call(&mut self, func_call: &crate::function_call::FunctionCall) -> Self::Output {
        todo!()
    }

    fn visit_unary_prefix(&mut self, expr: &crate::unary_prefix_expr::UnaryPrefixExpression) -> Self::Output {
        todo!()
    }

    fn visit_binary_expression(&mut self, expr: &crate::binary_expression::BinaryExpression) -> Self::Output {
        todo!()
    }

    fn visit_struct_member_access(&mut self, expr: &crate::struct_definition::StructMemberAccess) -> Self::Output {
        todo!()
    }
}