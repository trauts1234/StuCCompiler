use crate::{asm_gen_data::{AsmData, VariableAddress}, asm_generation::{asm_comment, asm_line, AssemblyOperand, LogicalRegister}, data_type::{base_type::BaseType, recursive_data_type::RecursiveDataType}, expression_visitors::{pop_struct_from_stack::PopStructFromStack, put_struct_on_stack::PutStructOnStack, reference_assembly_visitor::ReferenceVisitor}, memory_size::MemoryLayout};
use std::fmt::Write;
use unwrap_let::unwrap_let;
use super::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor};


/**
 * calculates the value of the expression and puts the scalar result in AX. does not leave anything on the stack
 * does not work with structs, as they are not scalar types
 * stack_data is modified
 */
pub struct ScalarInAccVisitor<'a>{
    pub(crate) asm_data: &'a AsmData,
    pub(crate) stack_data: &'a mut MemoryLayout//TODO is this right? 
}

impl<'a> ExprVisitor for ScalarInAccVisitor<'a> {
    type Output = String;

    fn visit_number_literal(&mut self, number: &crate::number_literal::NumberLiteral) -> Self::Output {
        let mut result = String::new();

        let reg_size = number.get_data_type().memory_size(self.asm_data);//decide how much storage is needed to temporarily store the constant
        asm_comment!(result, "reading number literal: {} via register {}", number.nasm_format(), LogicalRegister::ACC.generate_name(reg_size));

        asm_line!(result, "mov {}, {}", LogicalRegister::ACC.generate_name(reg_size), number.nasm_format());

        result
    }

    fn visit_variable(&mut self, var: &crate::declaration::MinimalDataVariable) -> Self::Output {
        let mut result = String::new();

        let my_type = var.accept(&mut GetDataTypeVisitor{asm_data: self.asm_data});

        if let RecursiveDataType::ARRAY { size:_, element:_ } = my_type {//is array
            //getting an array, decays to a pointer
            asm_comment!(result, "decaying array {} to pointer", var.name);
            let addr_asm = var.accept(&mut ReferenceVisitor{asm_data: self.asm_data, stack_data: self.stack_data});
            asm_line!(result, "{}", addr_asm);

        } else {
            let reg_size = my_type.memory_size(self.asm_data);//decide which register size is appropriate for this variable
            asm_comment!(result, "reading variable: {} to register {}", var.name, LogicalRegister::ACC.generate_name(reg_size));

            let result_reg = LogicalRegister::ACC.generate_name(reg_size);

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
        func_call.generate_assembly_scalar_return(self.asm_data, self.stack_data)
    }

    fn visit_unary_prefix(&mut self, expr: &crate::unary_prefix_expr::UnaryPrefixExpression) -> Self::Output {
        expr.generate_assembly(self.asm_data, self.stack_data)
    }

    fn visit_binary_expression(&mut self, expr: &crate::binary_expression::BinaryExpression) -> Self::Output {
        expr.generate_assembly(self.asm_data, self.stack_data)
    }

    fn visit_struct_member_access(&mut self, member_access: &crate::struct_definition::StructMemberAccess) -> Self::Output {
        let mut result = String::new();

        let member_name = member_access.get_member_name();
        unwrap_let!(RecursiveDataType::RAW(BaseType::STRUCT(original_struct_name)) = member_access.get_base_struct_tree().accept(&mut GetDataTypeVisitor{asm_data: self.asm_data}));

        let (member_decl, member_offset) = self.asm_data.get_struct(&original_struct_name).get_member_data(member_name);

        let result_reg = LogicalRegister::ACC.generate_name(member_decl.get_type().memory_size(self.asm_data));

        asm_line!(result, "{}", member_access.get_base_struct_tree().accept(&mut PutStructOnStack{asm_data: self.asm_data, stack_data: self.stack_data}));//generate struct that I am getting member of

        asm_comment!(result, "getting struct's member {}", member_name);

        asm_line!(result, "mov {}, [rax+{}]", result_reg, member_offset.size_bytes());//get member as an offset from the struct beginning

        asm_line!(result, "{}", member_access.accept(&mut PopStructFromStack{asm_data: self.asm_data}));//pop the struct if needed

        result
    }
}