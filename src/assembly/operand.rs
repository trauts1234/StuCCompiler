use crate::memory_size::MemoryLayout;

pub const PTR_SIZE: MemoryLayout = MemoryLayout::from_bytes(8);

/**
 * name of an actual register
 */
#[derive(Clone, Copy, Debug)]
pub enum AsmRegister {
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

#[derive(Clone, Debug)]
pub enum Operand {
    SubFromBP(MemoryLayout),
    AddToSP(MemoryLayout),
    ///memory layout stores how much to add to RBP to get the address of the value (remember 8 bytes for stack frame and 8 bytes for the return address)
    PreviousStackFrame(MemoryLayout),//not used much
    Register(AsmRegister),
    ImmediateValue(String),
    DerefAddress(AsmRegister),
    LabelAccess(String)
}

impl Operand {
    pub fn generate_name(&self, data_size: MemoryLayout) -> String {
        match self {
            Operand::SubFromBP(memory_layout) => format!("[rbp-{}]", memory_layout.size_bytes()),
            Operand::AddToSP(memory_layout) => format!("[rsp+{}]", memory_layout.size_bytes()),
            Operand::PreviousStackFrame(offset) => format!("[rbp+{}]", offset.size_bytes()),
            Operand::ImmediateValue(val) => val.to_string(),
            Operand::Register(physical_register) => physical_register.generate_name(data_size),
            Operand::DerefAddress(physical_register) => format!("[{}]", physical_register.generate_name(PTR_SIZE)),
            Operand::LabelAccess(label) => format!("[rel {}]", label),//[global_variable] gets the value
        }
    }
}

pub fn generate_param_reg(param_num: usize) -> AsmRegister {
    match param_num {
        0 => AsmRegister::_DI,
        1 => AsmRegister::_SI,
        2 => AsmRegister::_DX,
        3 => AsmRegister::_CX,
        4 => AsmRegister::R8,
        5 => AsmRegister::R9,
        6.. => panic!("this param should be on the stack.")
    }
}

impl AsmRegister {
    /**
     * generates the register for the accumulator
     */
    pub fn acc() -> Self {
        AsmRegister::_AX
    }
    /**
     * generates a register suitable for secondary storage of arithmetic
     */
    pub fn secondary() -> Self {
        AsmRegister::_CX
    }
    fn generate_name(&self, data_size: MemoryLayout) -> String {
        match (self, data_size.size_bytes()) {
            (AsmRegister::_SP, 8) => "rsp",
            (AsmRegister::_BP, 8) => "rbp",
            
            (AsmRegister::_AX, 8) => "rax",
            (AsmRegister::_BX, 8) => "rbx",
            (AsmRegister::_CX, 8) => "rcx",
            (AsmRegister::_DX, 8) => "rdx",
            (AsmRegister::_SI, 8) => "rsi",
            (AsmRegister::_DI, 8) => "rdi",
            (AsmRegister::R8,  8) => "r8",
            (AsmRegister::R9,  8) => "r9",

            (AsmRegister::_AX, 4) => "eax",
            (AsmRegister::_BX, 4) => "ebx",
            (AsmRegister::_CX, 4) => "ecx",
            (AsmRegister::_DX, 4) => "edx",
            (AsmRegister::_SI, 4) => "esi",
            (AsmRegister::_DI, 4) => "edi",
            (AsmRegister::R8,  4) => "r8d",
            (AsmRegister::R9,  4) => "r9d",

            (AsmRegister::_AX, 1) => "al",
            (AsmRegister::_BX, 1) => "bl",
            (AsmRegister::_CX, 1) => "cl",
            (AsmRegister::_DX, 1) => "dl",
            (AsmRegister::_SI, 1) => "sil",
            (AsmRegister::_DI, 1) => "dil",
            (AsmRegister::R8,  1) => "r8b",
            (AsmRegister::R9,  1) => "r9b",

            _ => panic!("invalid register-size combination for generating assembly")

        }.to_string()
    }
}