use colored::Colorize;
use memory_size::MemorySize;

use crate::{debugging::IRDisplay};

/**
 * name of an actual register
 */
#[derive(Clone, Copy, Debug)]
pub enum GPRegister {
    _AX,
    _BX,
    _CX,
    _DX,
    _SI,
    _DI,
    R8,
    R9,

    _SP,
    _BP,
}

/// Registers that hold floats
#[derive(Clone, Copy, Debug)]
pub enum MMRegister {
    XMM0,
    XMM1,
    XMM2,
    XMM3,
    XMM4,
    XMM5,
    XMM6,
    XMM7,
}

impl MMRegister {
    /**
     * generates the register for the floating-point accumulator
     */
    pub fn acc() -> Self {
        Self::XMM0
    }

    fn generate_variant_name(&self) -> &str {
        match self {
            MMRegister::XMM0 => "XMM0",
            MMRegister::XMM1 => "XMM1",
            MMRegister::XMM2 => "XMM2",
            MMRegister::XMM3 => "XMM3",
            MMRegister::XMM4 => "XMM4",
            MMRegister::XMM5 => "XMM5",
            MMRegister::XMM6 => "XMM6",
            MMRegister::XMM7 => "XMM7",
        }
    }

    pub fn generate_name(&self, data_size: MemorySize) -> String {
        match (self, data_size.size_bits()) {
            (Self::XMM0, 32) | (Self::XMM0, 64) => "xmm0",
            (Self::XMM1, 32) | (Self::XMM1, 64) => "xmm1",
            (Self::XMM2, 32) | (Self::XMM2, 64) => "xmm2",
            (Self::XMM3, 32) | (Self::XMM3, 64) => "xmm3",
            (Self::XMM4, 32) | (Self::XMM4, 64) => "xmm4",
            (Self::XMM5, 32) | (Self::XMM5, 64) => "xmm5",
            (Self::XMM6, 32) | (Self::XMM6, 64) => "xmm6",
            (Self::XMM7, 32) | (Self::XMM7, 64) => "xmm7",
            _ => panic!()
        }.to_string()
    }
}

impl GPRegister {
    /**
     * generates the register for the accumulator
     */
    pub fn acc() -> Self {
        GPRegister::_AX
    }
    /**
     * generates a register suitable for secondary storage of arithmetic
     */
    pub fn secondary() -> Self {
        GPRegister::_CX
    }

    pub fn third() -> Self {
        GPRegister::_DX
    }

    pub fn generate_name(&self, data_size: MemorySize) -> String {
        match (self, data_size.size_bytes()) {
            (GPRegister::_SP, 8) => "rsp",
            (GPRegister::_BP, 8) => "rbp",
            
            (GPRegister::_AX, 8) => "rax",
            (GPRegister::_BX, 8) => "rbx",
            (GPRegister::_CX, 8) => "rcx",
            (GPRegister::_DX, 8) => "rdx",
            (GPRegister::_SI, 8) => "rsi",
            (GPRegister::_DI, 8) => "rdi",
            (GPRegister::R8,  8) => "r8",
            (GPRegister::R9,  8) => "r9",

            (GPRegister::_AX, 4) => "eax",
            (GPRegister::_BX, 4) => "ebx",
            (GPRegister::_CX, 4) => "ecx",
            (GPRegister::_DX, 4) => "edx",
            (GPRegister::_SI, 4) => "esi",
            (GPRegister::_DI, 4) => "edi",
            (GPRegister::R8,  4) => "r8d",
            (GPRegister::R9,  4) => "r9d",

            (GPRegister::_AX, 2) => "ax",
            (GPRegister::_BX, 2) => "bx",
            (GPRegister::_CX, 2) => "cx",
            (GPRegister::_DX, 2) => "dx",
            (GPRegister::_SI, 2) => "si",
            (GPRegister::_DI, 2) => "di",
            (GPRegister::R8,  2) => "r8w",
            (GPRegister::R9,  2) => "r9w",

            (GPRegister::_AX, 1) => "al",
            (GPRegister::_BX, 1) => "bl",
            (GPRegister::_CX, 1) => "cl",
            (GPRegister::_DX, 1) => "dl",
            (GPRegister::_SI, 1) => "sil",
            (GPRegister::_DI, 1) => "dil",
            (GPRegister::R8,  1) => "r8b",
            (GPRegister::R9,  1) => "r9b",

            (reg, bytes) => panic!("cannot generate {} byte register for {:?}", bytes, reg)

        }.to_string()
    }

    fn generate_variant_name(&self) -> &str {
        match self {
            GPRegister::_AX => "_AX",
            GPRegister::_BX => "_BX",
            GPRegister::_CX => "_CX",
            GPRegister::_DX => "_DX",
            GPRegister::_SI => "_SI",
            GPRegister::_DI => "_DI",
            GPRegister::R8 => "R8",
            GPRegister::R9 => "R9",
            GPRegister::_SP => "_SP",
            GPRegister::_BP => "_BP",
        }
    }
}

impl IRDisplay for GPRegister {
    fn display_ir(&self) -> String {
        self.generate_variant_name().red().to_string()
    }
}
impl IRDisplay for MMRegister {
    fn display_ir(&self) -> String {
        self.generate_variant_name().red().to_string()
    }
}