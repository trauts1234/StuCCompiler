pub mod memory_operand;
pub mod register;
pub mod immediate;

use immediate::ImmediateValue;
use memory_operand::MemoryOperand;
use register::GPRegister;

use memory_size::MemorySize;

use crate::{assembly::operand::register::MMRegister, debugging::IRDisplay};

pub const PTR_SIZE: MemorySize = MemorySize::from_bytes(8);

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

#[derive(Clone)]
pub enum RegOrMem {
    GPReg(GPRegister),
    MMReg(MMRegister),
    Mem(MemoryOperand),
}

impl Into<Operand> for RegOrMem {
    fn into(self) -> Operand {
        match self {
            RegOrMem::GPReg(gpregister) => Operand::GPReg(gpregister),
            RegOrMem::MMReg(mmregister) => Operand::MMReg(mmregister),
            RegOrMem::Mem(memory_operand) => Operand::Mem(memory_operand),
        }
    }
}


impl Operand {
    pub fn generate_name(&self, data_size: MemorySize) -> String {
        match self {
            Operand::GPReg(register) => register.generate_name(data_size),
            Operand::MMReg(register) => register.generate_name(data_size),
            Operand::Mem(memory_operand) => memory_operand.generate_name(),
            Operand::Imm(immediate_value) => immediate_value.generate_name(),
        }
    }
}
impl RegOrMem {
    pub fn generate_name(&self, data_size: MemorySize) -> String {
        match self {
            RegOrMem::GPReg(register) => register.generate_name(data_size),
            RegOrMem::Mem(memory_operand) => memory_operand.generate_name(),
            _ => panic!()
        }
    }
}
impl IRDisplay for RegOrMem {
    fn display_ir(&self) -> String {
        match self {
            RegOrMem::GPReg(register) => register.display_ir(),
            RegOrMem::MMReg(register) => register.display_ir(),
            RegOrMem::Mem(memory_operand) => memory_operand.display_ir(),
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