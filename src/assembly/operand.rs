pub mod memory_operand;
pub mod register;
pub mod immediate;

use immediate::ImmediateValue;
use memory_operand::MemoryOperand;
use register::Register;

use memory_size::MemorySize;

use crate::debugging::IRDisplay;

pub const PTR_SIZE: MemorySize = MemorySize::from_bytes(8);

/**
 * enum storing any possible r/m or immediate operand
 */
#[derive(Clone, Debug)]
pub enum Operand {
    Reg(Register),
    Mem(MemoryOperand),
    Imm(ImmediateValue),
}

#[derive(Clone)]
pub enum RegOrMem {
    Reg(Register),
    Mem(MemoryOperand),
}

pub fn generate_param_reg(param_num: u64) -> Register {
    match param_num {
        0 => Register::_DI,
        1 => Register::_SI,
        2 => Register::_DX,
        3 => Register::_CX,
        4 => Register::R8,
        5 => Register::R9,
        6.. => panic!("this param should be on the stack.")
    }
}

impl Operand {
    pub fn generate_name(&self, data_size: MemorySize) -> String {
        match self {
            Operand::Reg(register) => register.generate_name(data_size),
            Operand::Mem(memory_operand) => memory_operand.generate_name(),
            Operand::Imm(immediate_value) => immediate_value.generate_name(),
        }
    }
}

impl RegOrMem {
    pub fn generate_name(&self, data_size: MemorySize) -> String {
        match self {
            RegOrMem::Reg(register) => register.generate_name(data_size),
            RegOrMem::Mem(memory_operand) => memory_operand.generate_name(),
        }
    }
}

impl IRDisplay for RegOrMem {
    fn display_ir(&self) -> String {
        match self {
            RegOrMem::Reg(register) => register.display_ir(),
            RegOrMem::Mem(memory_operand) => memory_operand.display_ir(),
        }
    }
}
impl IRDisplay for Operand {
    fn display_ir(&self) -> String {
        match self {
            Operand::Reg(register) => register.display_ir(),
            Operand::Mem(memory_operand) => memory_operand.display_ir(),
            Operand::Imm(immediate_value) => immediate_value.display_ir(),
        }
    }
}