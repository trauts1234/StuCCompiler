use crate::{asm_gen_data::{AsmData, VariableAddress}, asm_generation::{asm_comment, asm_line, LogicalRegister, RegisterName}, expression_visitors::reference_assembly_visitor::ReferenceVisitor};
use std::fmt::Write;
use super::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor};


/**
 * calculates the value of the expression and puts the scalar result in AX. does not leave anything on the stack
 * does not work with structs, as they are not scalar types
 */
pub struct ScalarInAccVisitor<'a>{
    pub(crate) asm_data: &'a AsmData
}

impl<'a> ExprVisitor for ScalarInAccVisitor<'a> {
    type Output = String;

    fn visit_number_literal(&mut self, number: &crate::number_literal::NumberLiteral) -> Self::Output {
        let mut result = String::new();

        let reg_size = &number.get_data_type().memory_size();//decide how much storage is needed to temporarily store the constant
        asm_comment!(result, "reading number literal: {} via register {}", number.nasm_format(), LogicalRegister::ACC.generate_reg_name(reg_size));

        asm_line!(result, "mov {}, {}", LogicalRegister::ACC.generate_reg_name(reg_size), number.nasm_format());

        result
    }

    fn visit_variable(&mut self, var: &crate::declaration::MinimalDataVariable) -> Self::Output {
        let mut result = String::new();

        let my_type = var.accept(&mut GetDataTypeVisitor{asm_data: self.asm_data});

        if my_type.is_array() {
            //getting an array, decays to a pointer
            asm_comment!(result, "decaying array {} to pointer", var.name);
            let addr_asm = var.accept(&mut ReferenceVisitor{asm_data: self.asm_data});
            asm_line!(result, "{}", addr_asm);

        } else {
            let reg_size = &my_type.memory_size();//decide which register size is appropriate for this variable
            asm_comment!(result, "reading variable: {} to register {}", var.name, LogicalRegister::ACC.generate_reg_name(reg_size));

            let result_reg = LogicalRegister::ACC.generate_reg_name(reg_size);

            match &self.asm_data.get_variable(&var.name).location {
                VariableAddress::CONSTANTADDRESS => 
                    asm_line!(result, "mov {}, [{}]", result_reg, var.name),
                VariableAddress::STACKOFFSET(stack_offset) => 
                    asm_line!(result, "mov {}, [rbp-{}]", result_reg, stack_offset.size_bytes())
            }
        }

        result
    }

    fn visit_string_literal(&mut self, string: &crate::string_literal::StringLiteral) -> Self::Output {
        format!("lea rax, [rel {}] ; decay string to char pointer", string.get_label())//warning: duplicated code from get address
    }

    fn visit_func_call(&mut self, func_call: &crate::function_call::FunctionCall) -> Self::Output {
        func_call.generate_assembly(self.asm_data)
    }

    fn visit_unary_prefix(&mut self, expr: &crate::unary_prefix_expr::UnaryPrefixExpression) -> Self::Output {
        expr.generate_assembly(self.asm_data)
    }

    fn visit_binary_expression(&mut self, expr: &crate::binary_expression::BinaryExpression) -> Self::Output {
        expr.generate_assembly(self.asm_data)
    }

    fn visit_struct_member_access(&mut self, expr: &crate::struct_definition::StructMemberAccess) -> Self::Output {
        todo!()
        //remember to deallocate struct
    }
}