use memory_size::MemoryLayout;

/**
 * name of an actual register
 */
#[derive(Clone, Copy, Debug)]
pub enum Register {
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

impl Register {
    /**
     * generates the register for the accumulator
     */
    pub fn acc() -> Self {
        Register::_AX
    }
    /**
     * generates a register suitable for secondary storage of arithmetic
     */
    pub fn secondary() -> Self {
        Register::_CX
    }
    
    pub fn generate_name(&self, data_size: MemoryLayout) -> String {
        match (self, data_size.size_bytes()) {
            (Register::_SP, 8) => "rsp",
            (Register::_BP, 8) => "rbp",
            
            (Register::_AX, 8) => "rax",
            (Register::_BX, 8) => "rbx",
            (Register::_CX, 8) => "rcx",
            (Register::_DX, 8) => "rdx",
            (Register::_SI, 8) => "rsi",
            (Register::_DI, 8) => "rdi",
            (Register::R8,  8) => "r8",
            (Register::R9,  8) => "r9",

            (Register::_AX, 4) => "eax",
            (Register::_BX, 4) => "ebx",
            (Register::_CX, 4) => "ecx",
            (Register::_DX, 4) => "edx",
            (Register::_SI, 4) => "esi",
            (Register::_DI, 4) => "edi",
            (Register::R8,  4) => "r8d",
            (Register::R9,  4) => "r9d",

            (Register::_AX, 2) => "ax",
            (Register::_BX, 2) => "bx",
            (Register::_CX, 2) => "cx",
            (Register::_DX, 2) => "dx",
            (Register::_SI, 2) => "si",
            (Register::_DI, 2) => "di",
            (Register::R8,  2) => "r8w",
            (Register::R9,  2) => "r9w",

            (Register::_AX, 1) => "al",
            (Register::_BX, 1) => "bl",
            (Register::_CX, 1) => "cl",
            (Register::_DX, 1) => "dl",
            (Register::_SI, 1) => "sil",
            (Register::_DI, 1) => "dil",
            (Register::R8,  1) => "r8b",
            (Register::R9,  1) => "r9b",

            _ => panic!("invalid register-size combination for generating assembly")

        }.to_string()
    }
}