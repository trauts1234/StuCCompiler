use crate::{asm_gen_data::AsmData, asm_generation::{asm_comment, asm_line, LogicalRegister, RegisterName, PTR_SIZE}, data_type::{base_type::BaseType, recursive_data_type::RecursiveDataType}, expression_visitors::{data_type_visitor::GetDataTypeVisitor, reference_assembly_visitor::ReferenceVisitor}, lexer::punctuator::Punctuator, memory_size::MemoryLayout};
use std::fmt::Write;
use unwrap_let::unwrap_let;
use super::expr_visitor::ExprVisitor;



pub struct PutStructOnStack<'a>{
    pub(crate) asm_data: &'a AsmData
}


/**
 * sets RAX to valid pointer to struct
 * always clones the struct
 */
impl<'a> ExprVisitor for PutStructOnStack<'a> {
    type Output = String;

    fn visit_number_literal(&mut self, _number: &crate::number_literal::NumberLiteral) -> Self::Output {
        panic!("tried to put struct on stack but found number literal")
    }

    fn visit_variable(&mut self, var: &crate::declaration::MinimalDataVariable) -> Self::Output {
        let mut result = String::new();

        asm_comment!(result, "cloning struct {}", var.name);
        asm_line!(result, "{}", var.accept(&mut ReferenceVisitor{asm_data:self.asm_data}));

        asm_line!(result, "{}", clone_struct_to_stack(var.accept(&mut GetDataTypeVisitor{asm_data:self.asm_data}).memory_size(self.asm_data)));

        result
    }

    fn visit_string_literal(&mut self, _string: &crate::string_literal::StringLiteral) -> Self::Output {
        panic!("tried to put struct on stack but found string literal");
    }

    fn visit_func_call(&mut self, func_call: &crate::function_call::FunctionCall) -> Self::Output {
        todo!("ABI compliant struct returning")
        //remember to set RAX to point to the struct (probably RSP)
    }

    /**
     * node: this does not allocate, since the pointer points to already-allocated memory
     */
    fn visit_unary_prefix(&mut self, expr: &crate::unary_prefix_expr::UnaryPrefixExpression) -> Self::Output {
        let mut result = String::new();
        assert!(*expr.get_operator() == Punctuator::ASTERISK);// unary prefix can only return a struct when it is a dereference operation
        
        asm_line!(result, "{}", expr.accept(&mut ReferenceVisitor{asm_data:self.asm_data}));

        asm_line!(result, "{}", clone_struct_to_stack(expr.accept(&mut GetDataTypeVisitor{asm_data:self.asm_data}).memory_size(self.asm_data)));

        result
    }

    fn visit_binary_expression(&mut self, _expr: &crate::binary_expression::BinaryExpression) -> Self::Output {
        panic!("tried to put struct on stack but found binary expression");
    }

    fn visit_struct_member_access(&mut self, member_access: &crate::struct_definition::StructMemberAccess) -> Self::Output {
        let mut result = String::new();

        let member_name = member_access.get_member_name();
        unwrap_let!(RecursiveDataType::RAW(BaseType::STRUCT(original_struct_name)) = member_access.get_base_struct_tree().accept(&mut GetDataTypeVisitor{asm_data: self.asm_data}));
        let member_data = self.asm_data.get_struct(&original_struct_name).get_member_data(member_name);

        asm_line!(result, "{}", member_access.get_base_struct_tree().accept(&mut PutStructOnStack{asm_data: self.asm_data}));//generate struct that I am getting member of

        asm_comment!(result, "increasing pointer to get index of member struct {}", member_data.0.get_name());

        asm_line!(result, "add rax, {}", member_data.1.size_bytes());//increase pointer to index of member

        result
    }
}

/**
 * clones the struct pointed to by acc onto the stack
 * moves acc to point to the start of the cloned struct
 */
fn clone_struct_to_stack(struct_size: MemoryLayout) -> String {
    let mut result = String::new();

    let acc_reg = LogicalRegister::ACC.generate_reg_name(&PTR_SIZE);

    asm_line!(result, "sub rsp, {}", struct_size.size_bytes());//allocate on the stack

    asm_line!(result, "mov rdi, rsp");//put destination in RDI
    asm_line!(result, "mov rsi, {}", acc_reg);//put source in RSI

    asm_line!(result, "mov rcx, {}", struct_size.size_bytes());//put number of bytes to copy in RCX

    asm_line!(result, "cld");//reset copy direction flag
    asm_line!(result, "rep movsb");//copy the data

    asm_line!(result, "mov {}, rsp", acc_reg);//point to the cloned struct

    result
}