use crate::{asm_boilerplate::cast_from_acc, asm_gen_data::AsmData, assembly::{assembly::Assembly, operand::{memory_operand::MemoryOperand, register::Register, Operand, RegOrMem}, operation::AsmOperation}, data_type::{base_type::BaseType, recursive_data_type::DataType}, expression_visitors::{put_struct_on_stack::CopyStructVisitor, reference_assembly_visitor::ReferenceVisitor}, memory_size::MemoryLayout};
use unwrap_let::unwrap_let;
use super::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor};


/**
 * calculates the value of the expression and puts the scalar result in AX. does not leave anything on the stack
 * does not work with structs, as they are not scalar types
 * stack_data is modified
 */
pub struct ScalarInAccVisitor<'a>{
    pub(crate) asm_data: &'a AsmData,
    pub(crate) stack_data: &'a mut MemoryLayout
}

impl<'a> ExprVisitor for ScalarInAccVisitor<'a> {
    type Output = Assembly;

    fn visit_number_literal(&mut self, number: &crate::number_literal::NumberLiteral) -> Self::Output {
        let mut result = Assembly::make_empty();

        let reg_size = number.get_data_type().memory_size(self.asm_data);//decide how much storage is needed to temporarily store the constant
        result.add_comment(format!("reading number literal: {}", number.nasm_format().0));

        result.add_instruction(AsmOperation::MOV {
            to: RegOrMem::Reg(Register::acc()),
            from: Operand::Imm(number.nasm_format()),
            size: reg_size,
        });

        result
    }

    fn visit_variable(&mut self, var: &crate::declaration::MinimalDataVariable) -> Self::Output {
        let mut result = Assembly::make_empty();

        let my_type = var.accept(&mut GetDataTypeVisitor{asm_data: self.asm_data});

        if let DataType::ARRAY {..} = my_type {//is array
            //getting an array, decays to a pointer
            result.add_comment(format!("decaying array {} to pointer", var.name));
            let addr_asm = var.accept(&mut ReferenceVisitor{asm_data: self.asm_data, stack_data: self.stack_data});
            result.merge(&addr_asm);

        } else {
            let reg_size = my_type.memory_size(self.asm_data);//decide which register size is appropriate for this variable
            result.add_comment(format!("reading variable: {}", var.name));

            result.add_instruction(AsmOperation::MOV {
                to: RegOrMem::Reg(Register::acc()),
                from: self.asm_data.get_variable(&var.name).location.clone(),
                size: reg_size,
            });
        }

        result
    }

    fn visit_string_literal(&mut self, string: &crate::string_literal::StringLiteral) -> Self::Output {
        let mut result = Assembly::make_empty();
        result.add_instruction(AsmOperation::LEA {
            to: Operand::Reg(Register::acc()),
            from: Operand::Mem(MemoryOperand::LabelAccess(string.get_label().to_string()))
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
        unwrap_let!(DataType::RAW(BaseType::STRUCT(original_struct_name)) = member_access.get_base_struct_tree().accept(&mut GetDataTypeVisitor{asm_data: self.asm_data}));

        let original_struct_definition = self.asm_data.get_struct(&original_struct_name);
        let (member_decl, member_offset) = original_struct_definition.get_member_data(member_name);

        *self.stack_data += original_struct_definition.calculate_size().unwrap();//allocate struct on stack
        let resultant_struct_location = Operand::Mem(MemoryOperand::SubFromBP(*self.stack_data));

        let struct_clone_asm = member_access.get_base_struct_tree().accept(&mut CopyStructVisitor{asm_data: self.asm_data, stack_data: self.stack_data, resultant_location: resultant_struct_location});
        result.merge(&struct_clone_asm);//generate struct that I am getting member of

        result.add_comment(format!("getting struct's member {}", member_name));

        //offset the start pointer to the address of the member
        result.add_instruction(AsmOperation::ADD {
            destination: RegOrMem::Reg(Register::acc()),
            increment: Operand::Imm(member_offset.as_imm()),
            data_type: DataType::RAW(BaseType::U64),
        });

        //dereference pointer
        result.add_instruction(AsmOperation::MOV {
            to: RegOrMem::Reg(Register::acc()),
            from: Operand::Mem(MemoryOperand::MemoryAddress{pointer_reg: Register::acc() }),
            size: member_decl.get_type().memory_size(self.asm_data),
        });

        //asm_line!(result, "{}", member_access.accept(&mut PopStructFromStack{asm_data: self.asm_data}));//pop the struct if needed

        result
    }
    
    fn visit_cast_expr(&mut self, expr: &crate::cast_expr::CastExpression) -> Self::Output {
        let mut result = Assembly::make_empty();
        let uncasted_asm = expr.get_uncasted_expr().accept(self);
        let uncasted_type = expr.get_uncasted_expr().accept(&mut GetDataTypeVisitor{asm_data:self.asm_data});
        //generate uncasted data
        result.merge(&uncasted_asm);
        //cast to required type
        result.merge(&cast_from_acc(&uncasted_type, expr.get_new_type(), self.asm_data));

        result
    }
}