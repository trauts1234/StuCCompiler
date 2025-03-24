use crate::{asm_gen_data::{AsmData, VariableAddress}, asm_generation::{self, LogicalRegister}, expression_visitors::{expr_visitor::ExprVisitor, put_scalar_in_acc::ScalarInAccVisitor}, lexer::punctuator::Punctuator};
use crate::asm_generation::RegisterName;


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
        todo!("function pointers")
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
        todo!()
    }
}