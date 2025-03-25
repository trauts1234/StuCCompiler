use crate::{asm_gen_data::{AsmData, VariableAddress}, asm_generation::{self, asm_comment, asm_line, LogicalRegister}, data_type::data_type::{Composite, DataType}, expression_visitors::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor, put_scalar_in_acc::ScalarInAccVisitor}, lexer::punctuator::Punctuator};
use crate::asm_generation::RegisterName;
use std::fmt::Write;
use unwrap_let::unwrap_let;

/**
 * puts the address of the visited Expression in RAX
 */
pub struct ReferenceVisitor<'a>{
    pub(crate) asm_data: &'a AsmData
}

impl<'a> ExprVisitor for ReferenceVisitor<'a> {
    type Output = String;

    fn visit_number_literal(&mut self, _number: &crate::number_literal::NumberLiteral) -> Self::Output {
        panic!("tried to get address of number literal")
    }

    fn visit_variable(&mut self, var: &crate::declaration::MinimalDataVariable) -> Self::Output {
        let ptr_reg = LogicalRegister::ACC.generate_reg_name(&asm_generation::PTR_SIZE);
        
        match &self.asm_data.get_variable(&var.name).location {
            VariableAddress::CONSTANTADDRESS => 
                format!("mov {}, {} ; getting address of global variable {}", ptr_reg, var.name, var.name),
            VariableAddress::STACKOFFSET(stack_offset) => 
                format!("lea {}, [rbp-{}] ; getting address of local variable {}", ptr_reg, stack_offset.size_bytes(), var.name)
        }
    }

    fn visit_string_literal(&mut self, string: &crate::string_literal::StringLiteral) -> Self::Output {
        format!("lea rax, [rel {}] ; get address of string literal", string.get_label())
    }

    fn visit_func_call(&mut self, _func_call: &crate::function_call::FunctionCall) -> Self::Output {
        panic!("cannot get address of function call")
    }

    fn visit_unary_prefix(&mut self, expr: &crate::unary_prefix_expr::UnaryPrefixExpression) -> Self::Output {
        //&*x == x
        assert!(*expr.get_operator() == Punctuator::ASTERISK);//must be address of a dereference

        let operand_asm = expr.get_operand().accept(&mut ScalarInAccVisitor{asm_data: self.asm_data});

        format!("{}; getting address of a dereference", operand_asm)
    }

    fn visit_binary_expression(&mut self, _expr: &crate::binary_expression::BinaryExpression) -> Self::Output {
        panic!("tried to get address of binary expression")
    }

    fn visit_struct_member_access(&mut self, expr: &crate::struct_definition::StructMemberAccess) -> Self::Output {
        let mut result = String::new();

        let member_name = expr.get_member_name();
        unwrap_let!(DataType::COMPOSITE(Composite { struct_name, modifiers }) = expr.accept(&mut GetDataTypeVisitor{asm_data: self.asm_data}));
        assert!(modifiers.modifiers_count() == 0);
        let member_data = self.asm_data.get_struct(&struct_name).get_member_data(member_name);

        asm_line!(result, "{}", expr.accept(&mut ReferenceVisitor{asm_data: self.asm_data}));//get address of the base struct

        asm_comment!(result, "increasing pointer to get address of member {}", member_data.0.get_name());

        asm_line!(result, "add rax, {}", member_data.1.size_bytes());//increase pointer to index of member

        result
    }
}