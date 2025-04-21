use crate::{asm_gen_data::AsmData, assembly::{assembly::Assembly, operand::{immediate::MemorySizeExt, memory_operand::MemoryOperand, register::Register, Operand, RegOrMem}, operation::AsmOperation}, data_type::{base_type::BaseType, recursive_data_type::DataType}, expression_operators::UnaryPrefixOperator, expression_visitors::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor, put_scalar_in_acc::ScalarInAccVisitor}, lexer::punctuator::Punctuator};
use unwrap_let::unwrap_let;
use memory_size::MemorySize;

/**
 * puts the address of the visited Expression in RAX
 */
pub struct ReferenceVisitor<'a>{
    pub(crate) asm_data: &'a AsmData,
    pub(crate) stack_data: &'a mut MemorySize
}

impl<'a> ExprVisitor for ReferenceVisitor<'a> {
    type Output = Assembly;

    fn visit_number_literal(&mut self, _number: &crate::number_literal::typed_value::NumberLiteral) -> Self::Output {
        panic!("tried to get address of number literal")
    }

    fn visit_variable(&mut self, var: &crate::declaration::MinimalDataVariable) -> Self::Output {

        let mut result = Assembly::make_empty();

        result.add_instruction(AsmOperation::LEA {
            to: RegOrMem::Reg(Register::acc()),
            from: self.asm_data.get_variable(&var.name).location.clone(),
        });

        result
    }

    fn visit_string_literal(&mut self, string: &crate::string_literal::StringLiteral) -> Self::Output {
        let mut result = Assembly::make_empty();

        result.add_instruction(AsmOperation::LEA {
            to: RegOrMem::Reg(Register::acc()),
            from: Operand::Mem(MemoryOperand::LabelAccess(string.get_label().to_string())),
        });

        result
    }

    fn visit_func_call(&mut self, _func_call: &crate::function_call::FunctionCall) -> Self::Output {
        panic!("cannot get address of function call")
    }

    fn visit_unary_prefix(&mut self, expr: &crate::unary_prefix_expr::UnaryPrefixExpression) -> Self::Output {
        let mut result = Assembly::make_empty();
        //&*x == x
        assert!(*expr.get_operator() == UnaryPrefixOperator::Dereference);//must be address of a dereference

        let operand_asm = expr.get_operand().accept(&mut ScalarInAccVisitor{asm_data: self.asm_data, stack_data: self.stack_data});

        result.add_comment("getting address of a dereference");
        result.merge(&operand_asm);

        result
    }

    fn visit_binary_expression(&mut self, _expr: &crate::binary_expression::BinaryExpression) -> Self::Output {
        panic!("tried to get address of binary expression")
    }

    fn visit_struct_member_access(&mut self, member_access: &crate::struct_definition::StructMemberAccess) -> Self::Output {
        let mut result = Assembly::make_empty();

        let member_name = member_access.get_member_name();
        //get the struct whose member I am getting
        unwrap_let!(DataType::RAW(BaseType::STRUCT(original_struct_name)) = member_access.get_base_struct_tree().accept(&mut GetDataTypeVisitor{asm_data: self.asm_data}));
        let member_data = self.asm_data.get_struct(&original_struct_name).get_member_data(member_name);

        //get address of the base struct
        let original_struct_asm = member_access.get_base_struct_tree().accept(&mut ReferenceVisitor{asm_data: self.asm_data, stack_data: self.stack_data});
        result.merge(&original_struct_asm);

        result.add_comment(format!("increasing pointer to get address of member {}", member_data.0.get_name()));

        //increase pointer to index of member
        result.add_instruction(AsmOperation::ADD {
            destination: RegOrMem::Reg(Register::acc()),
            increment: Operand::Imm(member_data.1.as_imm()),
            data_type: DataType::RAW(BaseType::U64),
        });

        result
    }
    
    fn visit_cast_expr(&mut self, _: &crate::cast_expr::CastExpression) -> Self::Output {
        panic!("cannot get address of a cast")
    }
}