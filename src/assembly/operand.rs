pub mod memory_operand;
pub mod register;
pub mod immediate;

use std::fmt::Display;

use colored::Colorize;
use immediate::ImmediateValue;
use memory_operand::MemoryOperand;
use register::GPRegister;

use memory_size::MemorySize;
use stack_management::{baked_stack_frame::BakedSimpleStackFrame, stack_item::StackItemKey};

use crate::{assembly::operand::register::MMRegister, debugging::IRDisplay};

pub const PTR_SIZE: MemorySize = MemorySize::from_bytes(8);
/// Alignment of the stack before calling a function in SysV ABI
/// 
/// Coincidentally also the alignment after a call and stack frame have been set up
pub const STACK_ALIGN: MemorySize = MemorySize::from_bytes(16);

#[derive(Clone)]
pub enum Storage {
    Stack(StackItemKey),
    Constant(ImmediateValue)
}

/**
 * enum storing any possible r/m or immediate operand
 */
#[derive(Clone, Debug)]
pub enum Operand {
    GPReg(GPRegister),
    MMReg(MMRegister),
    Mem(MemoryOperand),
    Imm(ImmediateValue),
}


impl Operand {
    pub fn generate_name(&self, data_size: MemorySize, stack: &BakedSimpleStackFrame) -> String {
        match self {
            Operand::GPReg(register) => register.generate_name(data_size),
            Operand::MMReg(register) => register.generate_name(data_size),
            Operand::Mem(memory_operand) => memory_operand.generate_name(stack),
            Operand::Imm(immediate_value) => immediate_value.generate_name(),
        }
    }
}
impl IRDisplay for Operand {
    fn display_ir(&self) -> String {
        match self {
            Operand::GPReg(register) => register.display_ir(),
            Operand::MMReg(register) => register.display_ir(),
            Operand::Mem(memory_operand) => memory_operand.display_ir(),
            Operand::Imm(immediate_value) => immediate_value.display_ir(),
        }
    }
}
impl Display for Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Storage::Stack(stack_item_key)=>format!("[{:?}]",stack_item_key),
            Storage::Constant(immediate_value) => immediate_value.generate_name(),
        }.blue())
    }
}