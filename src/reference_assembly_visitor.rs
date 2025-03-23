use crate::{asm_gen_data::VariableAddress, asm_generation::{self, asm_comment, asm_line, LogicalRegister}, expression::ExprVisitor, lexer::punctuator::Punctuator};
use crate::asm_generation::RegisterName;


pub struct ReferenceVisitor{
    assembly: Vec<String>
}

impl ReferenceVisitor {
    pub fn new() -> Self {
        ReferenceVisitor { assembly: Vec::new() }
    }

    pub fn get_assembly(&self) -> String {
        self.assembly.join("\n")
    }
}

impl ExprVisitor for ReferenceVisitor {
    type Output = ();

    fn visit_number_literal(&mut self, _number: &crate::number_literal::NumberLiteral) -> Self::Output {
        panic!("tried to get address of number literal")
    }

    fn visit_variable(&mut self, var: &crate::declaration::MinimalDataVariable, asm_data: &crate::asm_gen_data::AsmData) -> Self::Output {
        let ptr_reg = LogicalRegister::ACC.generate_reg_name(&asm_generation::PTR_SIZE);
        
        self.assembly.push(match &asm_data.get_variable(&var.name).location {
            VariableAddress::CONSTANTADDRESS => 
                format!("mov {}, {} ; getting address of global variable {}", ptr_reg, var.name, var.name),
            VariableAddress::STACKOFFSET(stack_offset) => 
                format!("lea {}, [rbp-{}] ; getting address of local variable {}", ptr_reg, stack_offset.size_bytes(), var.name)
        })
    }

    fn visit_string_literal(&mut self, string: &crate::string_literal::StringLiteral) -> Self::Output {
        self.assembly.push(format!("lea rax, [rel {}] ; get address of string literal", string.get_label()))
    }

    fn visit_func_call(&mut self, _func_call: &crate::function_call::FunctionCall, _asm_data: &crate::asm_gen_data::AsmData) -> Self::Output {
        todo!("function pointers")
    }

    fn visit_unary_prefix(&mut self, expr: &crate::unary_prefix_expr::UnaryPrefixExpression, asm_data: &crate::asm_gen_data::AsmData) -> Self::Output {
        //&*x == x
        assert!(*expr.get_operator() == Punctuator::ASTERISK);//must be address of a dereference

        self.assembly.push(
            format!("{}; getting address of a dereference", &expr.get_operand().put_value_in_accumulator(asm_data))
        );

        
    }

    fn visit_binary_expression(&mut self, expr: &crate::binary_expression::BinaryExpression, asm_data: &crate::asm_gen_data::AsmData) -> Self::Output {
        panic!("tried to get address of binary expression")
    }

    fn visit_struct_member_access(&mut self, expr: &crate::struct_definition::StructMemberAccess, asm_data: &crate::asm_gen_data::AsmData) -> Self::Output {
        todo!()
    }
}