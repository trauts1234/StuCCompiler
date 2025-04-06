use crate::memory_size::MemoryLayout;

pub const PTR_SIZE: MemoryLayout = MemoryLayout::from_bytes(8);

/**
 * stores register names based on what they are used for
 */
pub enum LogicalRegister{
    ACC,
    SECONDARY,
    THIRD,
}

/**
 * name of an actual register
 */
#[derive(Clone, Copy)]
pub enum PhysicalRegister {
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

#[derive(Clone)]
pub enum Operand {
    SubFromBP(MemoryLayout),
    AddToSP(MemoryLayout),
    Register(PhysicalRegister),
    ImmediateValue(String),
    DerefAddress(PhysicalRegister),
}

impl Operand {
    pub fn generate_name(&self, data_size: MemoryLayout) -> String {
        match self {
            Operand::SubFromBP(memory_layout) => format!("[rbp-{}]", memory_layout.size_bytes()),
            Operand::AddToSP(memory_layout) => format!("[rsp+{}]", memory_layout.size_bytes()),
            Operand::ImmediateValue(val) => val.to_string(),
            Operand::Register(physical_register) => physical_register.generate_name(data_size),
            Operand::DerefAddress(physical_register) => format!("[{}]", physical_register.generate_name(PTR_SIZE)),//memory addresses are always 64 bit
        }
    }
}

pub fn generate_param_reg(param_num: usize) -> PhysicalRegister {
    match param_num {
        0 => PhysicalRegister::_DI,
        1 => PhysicalRegister::_SI,
        2 => PhysicalRegister::_DX,
        3 => PhysicalRegister::_CX,
        4 => PhysicalRegister::R8,
        5 => PhysicalRegister::R9,
        6.. => panic!("this param should be on the stack.")
    }
}

impl LogicalRegister {
    /**
     * casts to physical register
     */
    pub fn base_reg(&self) -> PhysicalRegister {
        match self {
            LogicalRegister::ACC => PhysicalRegister::_AX,
            LogicalRegister::SECONDARY => PhysicalRegister::_CX,
            LogicalRegister::THIRD => PhysicalRegister::_DX,
        }
    }
}
impl PhysicalRegister {
    fn generate_name(&self, data_size: MemoryLayout) -> String {
        match (self, data_size.size_bytes()) {
            (PhysicalRegister::_SP, 8) => "rsp",
            (PhysicalRegister::_BP, 8) => "rbp",
            
            (PhysicalRegister::_AX, 8) => "rax",
            (PhysicalRegister::_BX, 8) => "rbx",
            (PhysicalRegister::_CX, 8) => "rcx",
            (PhysicalRegister::_DX, 8) => "rdx",
            (PhysicalRegister::_SI, 8) => "rsi",
            (PhysicalRegister::_DI, 8) => "rdi",
            (PhysicalRegister::R8,  8) => "r8",
            (PhysicalRegister::R9,  8) => "r9",

            (PhysicalRegister::_AX, 4) => "eax",
            (PhysicalRegister::_BX, 4) => "ebx",
            (PhysicalRegister::_CX, 4) => "ecx",
            (PhysicalRegister::_DX, 4) => "edx",
            (PhysicalRegister::_SI, 4) => "esi",
            (PhysicalRegister::_DI, 4) => "edi",
            (PhysicalRegister::R8,  4) => "r8d",
            (PhysicalRegister::R9,  4) => "r9d",

            (PhysicalRegister::_AX, 1) => "al",
            (PhysicalRegister::_BX, 1) => "bl",
            (PhysicalRegister::_CX, 1) => "cl",
            (PhysicalRegister::_DX, 1) => "dl",
            (PhysicalRegister::_SI, 1) => "sil",
            (PhysicalRegister::_DI, 1) => "dil",
            (PhysicalRegister::R8,  1) => "r8b",
            (PhysicalRegister::R9,  1) => "r9b",

            _ => panic!("invalid register-size combination for generating assembly")

        }.to_string()
    }
}