use std::fmt::Display;

use colored::Colorize;
use stack_management::{simple_stack_frame::SimpleStackFrame, stack_item::StackItemKey};

use crate::{asm_gen_data::{AsmData, GlobalAsmData}, assembly::{assembly::Assembly, operand::{memory_operand::MemoryOperand, register::GPRegister}, operation::AsmOperation}, data_type::recursive_data_type::DataType, expression::put_on_stack::PutOnStack, expression_visitors::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor, reference_assembly_visitor::ReferenceVisitor}};

#[derive(Clone, Debug)]
/**
 * stores enough data to know about a variable, using available context during assembly generation
 */
pub struct MinimalDataVariable {
    pub(crate) name: String
}

impl MinimalDataVariable {
    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_variable(self)
    }
}

impl PutOnStack for MinimalDataVariable {
    fn put_on_stack(&self, asm_data: &AsmData, stack: &mut SimpleStackFrame, global_asm_data: &GlobalAsmData) -> (Assembly, StackItemKey) {
        let mut result = Assembly::make_empty();
        let variable_size = self.accept(&mut GetDataTypeVisitor{asm_data}).memory_size(asm_data);
        let resultant_location = stack.allocate(variable_size);

        result.add_comment(format!("cloning variable {} to the stack at {:?}", self.name, resultant_location));

        //put pointer to variable in RAX
        let from_addr_asm = self.accept(&mut ReferenceVisitor{asm_data, stack_data: stack, global_asm_data});
        result.merge(&from_addr_asm);

        //memcpy the struct
        result.add_instruction(AsmOperation::MEMCPY {
            size: variable_size,
            from: MemoryOperand::MemoryAddress { pointer_reg: GPRegister::acc() },
            to: MemoryOperand::SubFromBP(resultant_location),
        });

        (result, resultant_location)
    }
}

/**
 * stores enough data to declare a variable:
 * name and data type
 */
#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub(crate) data_type: DataType,
    pub(crate) name: String,
}

impl Display for Declaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.data_type)
    }
}

impl Display for MinimalDataVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.blue())
    }
}