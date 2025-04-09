use crate::{asm_gen_data::AsmData, assembly::{assembly::Assembly, operand::{register::Register, Operand, RegOrMem, PTR_SIZE}, operation::AsmOperation}, data_type::{base_type::BaseType, recursive_data_type::DataType}, expression_visitors::{data_type_visitor::GetDataTypeVisitor, reference_assembly_visitor::ReferenceVisitor}, lexer::punctuator::Punctuator, memory_size::MemoryLayout};
use unwrap_let::unwrap_let;
use super::expr_visitor::ExprVisitor;



pub struct CopyStructVisitor<'a>{
    pub(crate) asm_data: &'a AsmData,
    pub(crate) stack_data: &'a mut MemoryLayout,
    pub(crate) resultant_location: Operand,
}


/**
 * sets RAX to valid pointer to struct
 * always clones the struct
 */
impl<'a> ExprVisitor for CopyStructVisitor<'a> {
    type Output = Assembly;

    fn visit_number_literal(&mut self, _number: &crate::number_literal::NumberLiteral) -> Self::Output {
        panic!("tried to put struct on stack but found number literal")
    }

    fn visit_variable(&mut self, var: &crate::declaration::MinimalDataVariable) -> Self::Output {
        let mut result = Assembly::make_empty();

        result.add_comment(format!("cloning struct {}", var.name));
        //put pointer to variable in RAX
        let from_addr_asm = var.accept(&mut ReferenceVisitor{asm_data:self.asm_data, stack_data: self.stack_data});
        result.merge(&from_addr_asm);

        let variable_size = var.accept(&mut GetDataTypeVisitor{asm_data:self.asm_data}).memory_size(self.asm_data);
        let struct_copy_asm = clone_struct_to_stack(variable_size, &self.resultant_location);
        result.merge(&struct_copy_asm);//memcpy the struct

        result
    }

    fn visit_string_literal(&mut self, _string: &crate::string_literal::StringLiteral) -> Self::Output {
        panic!("tried to put struct on stack but found string literal");
    }

    fn visit_func_call(&mut self, func_call: &crate::function_call::FunctionCall) -> Self::Output {
        let mut result = Assembly::make_empty();

        if let DataType::RAW(BaseType::STRUCT(struct_name)) = func_call.accept(&mut GetDataTypeVisitor{asm_data: self.asm_data}) {
            let struct_type = self.asm_data.get_struct(&struct_name);
            todo!("detect whether the struct is MEMORY or other, then allocate a hidden param or read from registers after function has been called. remember to align the stack")
        } else {
            panic!("Expected a struct type in function call");
        }

        result
    }

    /**
     * node: this does not allocate, since the pointer points to already-allocated memory
     */
    fn visit_unary_prefix(&mut self, expr: &crate::unary_prefix_expr::UnaryPrefixExpression) -> Self::Output {
        let mut result = Assembly::make_empty();
        assert!(*expr.get_operator() == Punctuator::ASTERISK);// unary prefix can only return a struct when it is a dereference operation
        
        let expr_addr_asm = expr.accept(&mut ReferenceVisitor{asm_data:self.asm_data, stack_data: self.stack_data});
        result.merge(&expr_addr_asm);

        let dereferenced_size = expr.accept(&mut GetDataTypeVisitor{asm_data:self.asm_data}).memory_size(self.asm_data);
        let struct_clone_asm = clone_struct_to_stack(dereferenced_size, &self.resultant_location);
        result.merge(&struct_clone_asm);

        result
    }

    fn visit_binary_expression(&mut self, _expr: &crate::binary_expression::BinaryExpression) -> Self::Output {
        panic!("tried to put struct on stack but found binary expression");
    }

    fn visit_struct_member_access(&mut self, member_access: &crate::struct_definition::StructMemberAccess) -> Self::Output {
        //this function handles getting struct members that are also structs themselves
        let mut result = Assembly::make_empty();

        let member_name = member_access.get_member_name();
        unwrap_let!(DataType::RAW(BaseType::STRUCT(original_struct_name)) = member_access.get_base_struct_tree().accept(&mut GetDataTypeVisitor{asm_data: self.asm_data}));
        let member_data = self.asm_data.get_struct(&original_struct_name).get_member_data(member_name);

        //generate struct that I am getting a member of
        let generate_struct_base = member_access.get_base_struct_tree().accept(&mut CopyStructVisitor{asm_data: self.asm_data, stack_data: self.stack_data, resultant_location: self.resultant_location.clone()});
        result.merge(&generate_struct_base);

        result.add_comment(format!("increasing pointer to get index of member struct {}", member_data.0.get_name()));

        //increase pointer to index of member
        result.add_instruction(AsmOperation::ADD {
            destination: RegOrMem::Reg(Register::acc()),
            increment: Operand::Imm(member_data.1.as_imm()),
            data_type: DataType::RAW(BaseType::U64),
        });

        result
    }
}

/**
 * clones the struct pointed to by acc onto the stack
 * moves acc to point to the start of the cloned struct
 */
fn clone_struct_to_stack(struct_size: MemoryLayout, resulatant_location: &Operand) -> Assembly {
    let mut result = Assembly::make_empty();

    
    //put destination in RDI
    result.add_instruction(AsmOperation::LEA {
        to: Operand::Reg(Register::_DI),
        from: resulatant_location.clone(),
    });
    //put source in RSI
    result.add_instruction(AsmOperation::MOV {
        to: RegOrMem::Reg(Register::_SI),
        from: Operand::Reg(Register::acc()),
        size: PTR_SIZE,
    });

    //clone struct
    result.add_instruction(AsmOperation::MEMCPY { size: struct_size });

    //point to the cloned struct
    result.add_instruction(AsmOperation::LEA {
        to: Operand::Reg(Register::acc()),
        from: resulatant_location.clone(),
    });

    result
}