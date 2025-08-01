use crate::{asm_gen_data::{AsmData, GetStruct}, assembly::{assembly::Assembly, operand::{immediate::ToImmediate, memory_operand::MemoryOperand, register::GPRegister, Operand, RegOrMem, PTR_SIZE}, operation::AsmOperation}, data_type::{base_type::{BaseType, IntegerType, ScalarType}, recursive_data_type::DataType}, expression::{unary_prefix_expr::UnaryPrefixExpression, unary_prefix_operator::UnaryPrefixOperator}, expression_visitors::{data_type_visitor::GetDataTypeVisitor, reference_assembly_visitor::ReferenceVisitor}, stack_allocation::StackAllocator, struct_member_access::StructMemberAccess};
use unwrap_let::unwrap_let;
use memory_size::MemorySize;
use super::expr_visitor::ExprVisitor;


/**
 * sets RAX to valid pointer to struct
 * always clones the struct
 */
pub struct CopyStructVisitor<'a>{
    pub(crate) asm_data: &'a AsmData,
    pub(crate) stack_data: &'a mut StackAllocator,
    pub(crate) resultant_location: Operand,
}


impl<'a> ExprVisitor for CopyStructVisitor<'a> {
    type Output = Assembly;

    fn visit_number_literal(&mut self, _number: &crate::number_literal::typed_value::NumberLiteral) -> Self::Output {
        panic!("tried to put struct on stack but found number literal")
    }

    fn visit_variable(&mut self, var: &crate::declaration::MinimalDataVariable) -> Self::Output {
        let mut result = Assembly::make_empty();

        result.add_comment(format!("cloning struct {}", var.name));
        //put pointer to variable in RAX
        let from_addr_asm = var.accept(&mut ReferenceVisitor{asm_data:self.asm_data, stack_data: self.stack_data});
        result.merge(&from_addr_asm);

        let variable_size = var.accept(&mut GetDataTypeVisitor{asm_data:self.asm_data}).memory_size(self.asm_data);
        unwrap_let!(Operand::Mem(resultant_mem_location) = self.resultant_location.clone());
        let struct_copy_asm = clone_struct_to_stack(variable_size, &resultant_mem_location);
        result.merge(&struct_copy_asm);//memcpy the struct

        result
    }

    fn visit_string_literal(&mut self, _string: &crate::string_literal::StringLiteral) -> Self::Output {
        panic!("tried to put struct on stack but found string literal");
    }

    fn visit_func_call(&mut self, func_call: &crate::function_call::FunctionCall) -> Self::Output {
        let mut result = Assembly::make_empty();

        if let DataType::RAW(BaseType::Struct(struct_name)) = func_call.accept(&mut GetDataTypeVisitor{asm_data: self.asm_data}) {
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
    fn visit_unary_prefix(&mut self, expr: &UnaryPrefixExpression) -> Self::Output {
        let mut result = Assembly::make_empty();
        assert!(*expr.get_operator() == UnaryPrefixOperator::Dereference);// unary prefix can only return a struct when it is a dereference operation
        
        let expr_addr_asm = expr.accept(&mut ReferenceVisitor{asm_data:self.asm_data, stack_data: self.stack_data});
        result.merge(&expr_addr_asm);

        let dereferenced_size = expr.accept(&mut GetDataTypeVisitor{asm_data:self.asm_data}).memory_size(self.asm_data);
        unwrap_let!(Operand::Mem(resultant_mem_location) = self.resultant_location.clone());
        let struct_clone_asm = clone_struct_to_stack(dereferenced_size, &resultant_mem_location);
        result.merge(&struct_clone_asm);

        result
    }

    fn visit_unary_postfix(&mut self, _expr: &crate::expression::unary_postfix_expression::UnaryPostfixExpression) -> Self::Output {
        panic!("tried to put struct on stack but found a unary postfix expression")
    }

    fn visit_binary_expression(&mut self, _expr: &crate::binary_expression::BinaryExpression) -> Self::Output {
        panic!("tried to put struct on stack but found binary expression");
    }

    fn visit_struct_member_access(&mut self, member_access: &StructMemberAccess) -> Self::Output {
        //this function handles getting struct members that are also structs themselves
        let mut result = Assembly::make_empty();

        let member_name = member_access.get_member_name();
        unwrap_let!(DataType::RAW(BaseType::Struct(original_struct_name)) = member_access.get_base_struct_tree().accept(&mut GetDataTypeVisitor{asm_data: self.asm_data}));
        let member_data = self.asm_data.get_struct(&original_struct_name).get_member_data(member_name);

        //generate struct that I am getting a member of
        let generate_struct_base = member_access.get_base_struct_tree().accept(&mut CopyStructVisitor{asm_data: self.asm_data, stack_data: self.stack_data, resultant_location: self.resultant_location.clone()});
        result.merge(&generate_struct_base);

        result.add_comment(format!("increasing pointer to get index of member struct {}", member_data.0.name));

        //increase pointer to index of member
        result.add_instruction(AsmOperation::ADD {
            increment: Operand::Imm(member_data.1.as_imm()),
            data_type: ScalarType::Integer(IntegerType::U64),
        });

        result
    }
    
    fn visit_cast_expr(&mut self, _: &crate::cast_expr::CastExpression) -> Self::Output {
        panic!("cannot cast to struct")
    }
    
    fn visit_sizeof(&mut self, _: &crate::expression::sizeof_expression::SizeofExpr) -> Self::Output {
        panic!("sizeof never returns a struct")
    }
}

/**
 * clones the struct pointed to by acc onto the stack
 * moves acc to point to the start of the cloned struct
 */
fn clone_struct_to_stack(struct_size: MemorySize, destination: &MemoryOperand) -> Assembly {
    let mut result = Assembly::make_empty();

    //put source in RSI
    result.add_instruction(AsmOperation::MOV {
        to: RegOrMem::GPReg(GPRegister::_SI),
        from: Operand::GPReg(GPRegister::acc()),
        size: PTR_SIZE,
    });
    
    //put destination in RDI
    result.add_instruction(AsmOperation::LEA {
        from: destination.clone(),
    });
    result.add_instruction(AsmOperation::MOV {
        to: RegOrMem::GPReg(GPRegister::_DI),
        from: Operand::GPReg(GPRegister::acc()),
        size: PTR_SIZE
    });

    //clone struct
    result.add_instruction(AsmOperation::MEMCPY { size: struct_size });

    //point to the cloned struct
    result.add_instruction(AsmOperation::LEA {
        from: destination.clone(),
    });

    result
}