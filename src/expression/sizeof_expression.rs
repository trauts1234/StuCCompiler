use colored::Colorize;
use memory_size::MemorySize;
use crate::{asm_gen_data::AsmData, assembly::{assembly::Assembly, operand::{immediate::ToImmediate, memory_operand::MemoryOperand, register::GPRegister, Operand, RegOrMem, PTR_SIZE}, operation::AsmOperation}, data_type::recursive_data_type::DataType, debugging::ASTDisplay, expression::put_on_stack::PutOnStack, expression_visitors::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor}};

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
            to: RegOrMem::GPReg(GPRegister::acc()),
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

impl PutOnStack for SizeofExpr {
    fn put_on_stack(&self, asm_data: &AsmData, stack: &mut stack_management::simple_stack_frame::SimpleStackFrame, global_asm_data: &crate::asm_gen_data::GlobalAsmData) -> (Assembly, stack_management::stack_item::StackItemKey) {
        let size = self.get_result(asm_data);
        let mut asm = Assembly::make_empty();
        let resultant_location = stack.allocate(PTR_SIZE);

        asm.add_instruction(AsmOperation::MOV {
            to: RegOrMem::GPReg(GPRegister::acc()),
            from: Operand::Imm(size.as_imm()),
            size: PTR_SIZE,//standard actually says it should be a size_t
        });
        asm.add_instruction(AsmOperation::MOV {
            to: RegOrMem::Mem(MemoryOperand::SubFromBP(resultant_location)),
            from: Operand::GPReg(GPRegister::acc()),
            size: PTR_SIZE,//standard actually says it should be a size_t
        });

        (asm, resultant_location)
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