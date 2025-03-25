use crate::{asm_gen_data::AsmData, asm_generation::{asm_comment, asm_line}, data_type::data_type::{Composite, DataType}, expression_visitors::{data_type_visitor::GetDataTypeVisitor, reference_assembly_visitor::ReferenceVisitor}, lexer::punctuator::Punctuator};
use std::fmt::Write;
use unwrap_let::unwrap_let;
use super::expr_visitor::ExprVisitor;



pub struct PutStructOnStack<'a>{
    pub(crate) asm_data: &'a AsmData
}


/**
 * sets RAX to valid pointer to struct
 * allocates on stack if required
 */
impl<'a> ExprVisitor for PutStructOnStack<'a> {
    type Output = String;

    fn visit_number_literal(&mut self, _number: &crate::number_literal::NumberLiteral) -> Self::Output {
        panic!("tried to put struct on stack but found number literal")
    }

    /**
     * note: this does not allocate, since the variable already exists
     * warning: you need to deallocate the stack allocated by this assembly, if it does allocate
     */
    fn visit_variable(&mut self, var: &crate::declaration::MinimalDataVariable) -> Self::Output {
        let mut result = String::new();

        asm_comment!(result, "getting address of struct {} instead of copying to stack", var.name);
        asm_line!(result, "{}", var.accept(&mut ReferenceVisitor{asm_data:self.asm_data}));

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
        assert!(*expr.get_operator() == Punctuator::ASTERISK);// unary prefix can only return a struct when it is a dereference operation
        
        expr.accept(&mut ReferenceVisitor{asm_data:self.asm_data})//put address in RAX, since the data is already allocated
    }

    fn visit_binary_expression(&mut self, _expr: &crate::binary_expression::BinaryExpression) -> Self::Output {
        panic!("tried to put struct on stack but found binary expression");
    }

    fn visit_struct_member_access(&mut self, expr: &crate::struct_definition::StructMemberAccess) -> Self::Output {
        let mut result = String::new();

        let member_name = expr.get_member_name();
        unwrap_let!(DataType::COMPOSITE(Composite { struct_name, modifiers }) = expr.accept(&mut GetDataTypeVisitor{asm_data: self.asm_data}));
        assert!(modifiers.len() == 0);
        let member_data = self.asm_data.get_struct(&struct_name).get_member_data(member_name);

        asm_line!(result, "{}", expr.accept(&mut PutStructOnStack{asm_data: self.asm_data}));//generate struct that I am getting member of

        asm_comment!(result, "increasing pointer to get index of member {}", member_data.0.get_name());

        asm_line!(result, "add rax, {}", member_data.1.size_bytes());//increase pointer to index of member

        result
    }
}