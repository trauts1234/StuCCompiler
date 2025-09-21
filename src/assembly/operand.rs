pub mod memory_operand;
pub mod register;
pub mod immediate;

use std::fmt::Display;

use colored::Colorize;
use memory_size::MemorySize;
use stack_management::stack_item::StackItemKey;

use crate::number_literal::typed_value::NumberLiteral;

pub const PTR_SIZE: MemorySize = MemorySize::from_bytes(8);
/// Alignment of the stack before calling a function in SysV ABI
/// 
/// Coincidentally also the alignment after a call and stack frame have been set up
pub const STACK_ALIGN: MemorySize = MemorySize::from_bytes(16);

#[derive(Clone)]
pub enum Storage {
    Stack(StackItemKey),
    StackWithOffset{stack: StackItemKey, offset: MemorySize},
    Constant(NumberLiteral),
    /// Dereferences the pointer at `self.0`
    IndirectAddress(StackItemKey),
}

impl Display for Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Storage::Stack(stack_item_key)=>format!("[{:?}]",stack_item_key),
            Storage::StackWithOffset{stack,offset}=>format!("[{:?} + {}]",stack,offset),
            Storage::Constant(immediate_value)=>immediate_value.to_string(),
            Storage::IndirectAddress(stack_item_key) => format!("[[{:?}]]", stack_item_key),
        }.blue())
    }
}