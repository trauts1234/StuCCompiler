pub mod register;
pub mod immediate;

use std::fmt::Debug;

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

impl Debug for Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Storage::Stack(stack_item_key)=>format!("[{:?}]",stack_item_key),
            Storage::StackWithOffset{stack,offset}=>format!("[{:?} + {}]",stack,offset),
            Storage::Constant(immediate_value)=>immediate_value.to_string(),
            Storage::IndirectAddress(stack_item_key) => format!("[[{:?}]]", stack_item_key),
        }.blue())
    }
}

impl Into<IROperand> for Storage {
    fn into(self) -> IROperand {
        match self {
            Storage::Stack(stack_item_key) => IROperand::Memory(IRMemOperand::Stack { base: stack_item_key}),
            Storage::StackWithOffset { stack, offset } => IROperand::Memory(IRMemOperand::OffsetAddress { base: Box::new(IRMemOperand::Stack { base: stack}), displacement: offset }),
            Storage::Constant(number_literal) => IROperand::Constant(number_literal),
            Storage::IndirectAddress(stack_item_key) => IROperand::Memory(IRMemOperand::IndirectAddress { pointer_location: Box::new(IRMemOperand::Stack { base: stack_item_key})}),
        }
    }
}
impl TryInto<IRMemOperand> for Storage {
    type Error = ();

    fn try_into(self) -> Result<IRMemOperand, Self::Error> {
        match self {
            Storage::Stack(stack_item_key) => Ok(IRMemOperand::Stack { base: stack_item_key}),
            Storage::StackWithOffset { stack, offset } => Ok(IRMemOperand::OffsetAddress { base: Box::new(IRMemOperand::Stack { base: stack}), displacement: offset }),
            Storage::Constant(_) => Err(()),
            Storage::IndirectAddress(stack_item_key) => Ok(IRMemOperand::IndirectAddress { pointer_location: Box::new(IRMemOperand::Stack { base: stack_item_key})}),
        }
    }
}

/// Operand for the IR that relates to a location in memory
#[derive(Clone)]
pub enum IRMemOperand {
    Stack {base: StackItemKey},
    IndirectAddress{ pointer_location: Box<IRMemOperand>},
    OffsetAddress {
        /// Not a pointer. This is the actual location
        base: Box<IRMemOperand>,
        displacement: MemorySize
    },
    //TODO label access (no displacement?)
}

/// Operand for the IR that relates to something with value
#[derive(Clone)]
pub enum IROperand {
    Memory(IRMemOperand),
    Constant(NumberLiteral),
}

impl Debug for IRMemOperand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stack { base } => write!(f, "{:?}", base),
            Self::IndirectAddress { pointer_location} => write!(f, "[{:?}]", pointer_location),
            Self::OffsetAddress { base, displacement } => write!(f, "[{} + &{:?}]", displacement.size_bytes(), base)
        }
    }
}

impl Debug for IROperand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Memory(arg0) => write!(f, "{:?}", arg0),
            Self::Constant(arg0) => write!(f, "{:?}", arg0),
        }
    }
}