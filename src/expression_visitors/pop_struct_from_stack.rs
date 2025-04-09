use crate::{asm_gen_data::AsmData, assembly::{assembly::Assembly, operand::{Operand, AsmRegister}, operation::AsmOperation}, data_type::{base_type::BaseType, recursive_data_type::DataType}, expression_visitors::data_type_visitor::GetDataTypeVisitor, lexer::punctuator::Punctuator};
use super::expr_visitor::ExprVisitor;


/**
 * pops the struct specified from the stack
 * guaranteed to not modify RAX
 */
pub struct PopStructFromStack<'a>{
    pub(crate) asm_data: &'a AsmData
}

impl<'a> ExprVisitor for PopStructFromStack<'a> {
    type Output = Assembly;

    fn visit_number_literal(&mut self, _number: &crate::number_literal::NumberLiteral) -> Self::Output {
        panic!()
    }

    fn visit_variable(&mut self, var: &crate::declaration::MinimalDataVariable) -> Self::Output {
        let mut result = Assembly::make_empty();

        result.add_comment(format!("popping struct {}", var.name));

        let struct_size = var.accept(&mut GetDataTypeVisitor{asm_data:self.asm_data}).memory_size(self.asm_data);

        result.add_instruction(AsmOperation::ADD {
            destination: Operand::Register(AsmRegister::_SP),
            increment: Operand::ImmediateValue(struct_size.size_bytes().to_string()),
            data_type: DataType::RAW(BaseType::U64),
        });

        result
    }

    fn visit_string_literal(&mut self, _string: &crate::string_literal::StringLiteral) -> Self::Output {
        panic!();
    }

    fn visit_func_call(&mut self, func_call: &crate::function_call::FunctionCall) -> Self::Output {
        let mut result = Assembly::make_empty();

        let return_value = func_call.accept(&mut GetDataTypeVisitor{asm_data: self.asm_data});
        let return_size = return_value.memory_size(self.asm_data);

        let callee_name = &func_call.get_callee_decl().function_name;

        result.add_commented_instruction(AsmOperation::ADD {
            destination: Operand::Register(AsmRegister::_SP),
            increment: Operand::ImmediateValue(return_size.size_bytes().to_string()),
            data_type: DataType::RAW(BaseType::U64),
        }, format!("deallocate a struct returned from a function call to {}", callee_name));

        result
    }

    fn visit_unary_prefix(&mut self, expr: &crate::unary_prefix_expr::UnaryPrefixExpression) -> Self::Output {
        let mut result = Assembly::make_empty();

        assert!(*expr.get_operator() == Punctuator::ASTERISK);//must be dereference

        let underlying_size = expr.accept(&mut GetDataTypeVisitor{asm_data:self.asm_data}).memory_size(self.asm_data);

        result.add_instruction(AsmOperation::ADD {
            destination: Operand::Register(AsmRegister::_SP),
            increment: Operand::ImmediateValue(underlying_size.size_bytes().to_string()),
            data_type: DataType::RAW(BaseType::U64),
        });

        result
    }

    fn visit_binary_expression(&mut self, _expr: &crate::binary_expression::BinaryExpression) -> Self::Output {
        panic!();
    }

    fn visit_struct_member_access(&mut self, expr: &crate::struct_definition::StructMemberAccess) -> Self::Output {
        expr.get_base_struct_tree().accept(&mut PopStructFromStack{asm_data:self.asm_data})//pop base struct that I got my struct from, as that may allocate: foo().x_member
    }
}