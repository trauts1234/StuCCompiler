use colored::Colorize;
use memory_size::MemorySize;
use crate::{asm_gen_data::AsmData, assembly::{assembly::Assembly, operand::{immediate::MemorySizeExt, register::GPRegister, Operand, GPRegOrMem, PTR_SIZE}, operation::AsmOperation}, data_type::recursive_data_type::DataType, debugging::ASTDisplay, expression_visitors::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor}};

use super::expression::Expression;

#[derive(Clone, Debug)]
pub enum SizeofExpr {
    SizeofExpression(Box<Expression>),
    SizeofType(DataType)
}

impl SizeofExpr {
    pub fn generate_assembly(&self, asm_data: &AsmData) -> Assembly {
        let mut result = Assembly::make_empty();

        let size = self.get_result(asm_data);

        result.add_instruction(AsmOperation::MOV {
            to: GPRegOrMem::Reg(GPRegister::acc()),
            from: Operand::Imm(size.as_imm()),
            size: PTR_SIZE,//standard actually says it should be a size_t
        });

        result
    }

    /// evaluates the expression by calculating the size 
    pub fn get_result(&self, asm_data: &AsmData) -> MemorySize {
        match self {
            SizeofExpr::SizeofExpression(x) => x.accept(&mut GetDataTypeVisitor {asm_data}).memory_size(asm_data),
            SizeofExpr::SizeofType(x) => x.memory_size(asm_data)
        }
    }

    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_sizeof(&self)
    }
}

impl ASTDisplay for SizeofExpr {
    fn display_ast(&self, f: &mut crate::debugging::TreeDisplayInfo) {
        f.write(&"sizeof".yellow().to_string());
        f.indent();
        match self {
            SizeofExpr::SizeofExpression(expression) => expression.display_ast(f),
            SizeofExpr::SizeofType(data_type) => f.write(&format!("{}", data_type)),
        };
        f.dedent();
    }
}