use std::fmt::Display;

use colored::Colorize;

use crate::{assembly::{assembly::IRCode, operand::{IRMemOperand, Storage, PTR_SIZE}, operation::IROperation}, data_type::recursive_data_type::DataType, expression_visitors::expr_visitor::ExprVisitor, generate_ir_traits::{GenerateIR, GetAddress, GetType}};

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

impl GetType for MinimalDataVariable {
    fn get_type(&self, asm_data: &crate::asm_gen_data::AsmData) -> DataType {
        asm_data.get_variable(&self.name).data_type.clone()
    }
}
impl GetAddress for MinimalDataVariable {
    fn get_address(&self, asm_data: &crate::asm_gen_data::AsmData, stack_data: &mut stack_management::simple_stack_frame::SimpleStackFrame, global_asm_data: &crate::asm_gen_data::GlobalAsmData) -> (crate::assembly::assembly::IRCode, stack_management::stack_item::StackItemKey) {
        let ptr = stack_data.allocate(PTR_SIZE);
        let mut result = IRCode::make_empty();

        let location = asm_data.get_variable(&self.name).location.clone();

        println!("var {} at {:?}", self.name, location);

        result.add_instruction(IROperation::LEA {
            from: location.try_into().unwrap(),
            to: IRMemOperand::Stack { base: ptr },
        });

        (result, ptr)
    }
}
impl GenerateIR for MinimalDataVariable {
    fn generate_ir(&self, asm_data: &crate::asm_gen_data::AsmData, stack_data: &mut stack_management::simple_stack_frame::SimpleStackFrame, global_asm_data: &crate::asm_gen_data::GlobalAsmData) -> (IRCode, Option<stack_management::stack_item::StackItemKey>) {
        let var_size = self.get_type(asm_data).memory_size(asm_data);
        let var_result = stack_data.allocate(var_size);
        let mut result = IRCode::make_empty();

        let var_data = &asm_data.get_variable(&self.name);
        if matches!(var_data.data_type, DataType::ARRAY {..} | DataType::UNKNOWNSIZEARRAY {..}) {
            let (ir, dest) = self.get_address(asm_data, stack_data, global_asm_data);
            return (ir, Some(dest));//array decays to pointer
        }

        result.add_commented_instruction(IROperation::MOV {
            from: var_data.location.clone().into(),
            to: IRMemOperand::Stack { base: var_result },
            size: var_size,
        }, format!("cloning var {}", self.name));

        (result, Some(var_result))
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