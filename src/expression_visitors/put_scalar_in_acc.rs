use crate::{asm_gen_data::AsmData, assembly::{assembly::Assembly, operand::{LogicalRegister, Operand}, operation::AsmOperation}, data_type::{base_type::BaseType, recursive_data_type::RecursiveDataType}, expression_visitors::{put_struct_on_stack::CopyStructVisitor, reference_assembly_visitor::ReferenceVisitor}, memory_size::MemoryLayout};
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
    type Output = Assembly;

    fn visit_number_literal(&mut self, number: &crate::number_literal::NumberLiteral) -> Self::Output {
        let mut result = Assembly::make_empty();

        let reg_size = number.get_data_type().memory_size(self.asm_data);//decide how much storage is needed to temporarily store the constant
        result.add_comment(format!("reading number literal: {}", number.nasm_format()));

        result.add_instruction(AsmOperation::MOV {
            to: Operand::Register(LogicalRegister::ACC.base_reg()),
            from: Operand::ImmediateValue(number.nasm_format()),
            size: reg_size,
        });

        result
    }

    fn visit_variable(&mut self, var: &crate::declaration::MinimalDataVariable) -> Self::Output {
        let mut result = Assembly::make_empty();

        let my_type = var.accept(&mut GetDataTypeVisitor{asm_data: self.asm_data});

        if let RecursiveDataType::ARRAY {..} = my_type {//is array
            //getting an array, decays to a pointer
            result.add_comment(format!("decaying array {} to pointer", var.name));
            let addr_asm = var.accept(&mut ReferenceVisitor{asm_data: self.asm_data, stack_data: self.stack_data});
            result.merge(&addr_asm);

        } else {
            let reg_size = my_type.memory_size(self.asm_data);//decide which register size is appropriate for this variable
            result.add_comment(format!("reading variable: {}", var.name));

            result.add_instruction(AsmOperation::MOV {
                to: Operand::Register(LogicalRegister::ACC.base_reg()),
                from: self.asm_data.get_variable(&var.name).location.clone(),
                size: reg_size,
            });
        }

        result
    }

    fn visit_string_literal(&mut self, string: &crate::string_literal::StringLiteral) -> Self::Output {
        let mut result = Assembly::make_empty();
        result.add_instruction(AsmOperation::LEA {
            to: Operand::Register(LogicalRegister::ACC.base_reg()),
            from: Operand::LabelAccess(string.get_label().to_string())
        });//warning: duplicated code from get address

        result
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
        let mut result = Assembly::make_empty();

        let member_name = member_access.get_member_name();
        unwrap_let!(RecursiveDataType::RAW(BaseType::STRUCT(original_struct_name)) = member_access.get_base_struct_tree().accept(&mut GetDataTypeVisitor{asm_data: self.asm_data}));

        let original_struct_definition = self.asm_data.get_struct(&original_struct_name);
        let (member_decl, member_offset) = original_struct_definition.get_member_data(member_name);

        *self.stack_data += original_struct_definition.calculate_size().unwrap();//allocate struct on stack
        let resultant_struct_location = Operand::SubFromBP(*self.stack_data);

        let struct_clone_asm = member_access.get_base_struct_tree().accept(&mut CopyStructVisitor{asm_data: self.asm_data, stack_data: self.stack_data, resultant_location: resultant_struct_location});
        result.merge(&struct_clone_asm);//generate struct that I am getting member of

        result.add_comment(format!("getting struct's member {}", member_name));

        //offset the start pointer to the address of the member
        result.add_instruction(AsmOperation::ADD {
            destination: Operand::Register(LogicalRegister::ACC.base_reg()),
            increment: Operand::ImmediateValue(member_offset.size_bytes().to_string()),
            data_type: RecursiveDataType::RAW(BaseType::U64),
        });

        //dereference pointer
        result.add_instruction(AsmOperation::MOV {
            to: Operand::Register(LogicalRegister::ACC.base_reg()),
            from: Operand::DerefAddress(LogicalRegister::ACC.base_reg()),
            size: member_decl.get_type().memory_size(self.asm_data),
        });

        //asm_line!(result, "{}", member_access.accept(&mut PopStructFromStack{asm_data: self.asm_data}));//pop the struct if needed

        result
    }
}