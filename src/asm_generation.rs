use crate::memory_size::MemoryLayout;

pub const PTR_SIZE: MemoryLayout = MemoryLayout::from_bytes(8);


/**
 * this funky macro runs like writeln!(result, "text {}", foo), but
 * requires no unwrap and
 * automatically manages newlines
 */
macro_rules! asm_line {
    ($dest:expr, $($arg:tt)*) => {{
        let s = format!($($arg)*);
        if s.trim().is_empty() {
            // blank or whitespace - do not save anything
        } else if s.ends_with('\n') {
            write!($dest, "{}", s).unwrap()
        } else {
            writeln!($dest, "{}", s).unwrap()
        }
    }};
}
pub(crate) use asm_line;

macro_rules! asm_comment {
    ($dest:expr, $($arg:tt)*) => {{
        let s = format!($($arg)*);
        if s.ends_with('\n') {
            write!($dest, ";{}", s).unwrap()
        } else {
            writeln!($dest, ";{}", s).unwrap()
        }
    }};
}
pub(crate) use asm_comment;

/**
 * trait to allow logical or physical registers to generate a register name
 */
pub trait RegisterName {
    fn generate_reg_name(&self, data_size: &MemoryLayout) -> String;
}

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

}

pub fn generate_param_reg(param_num: usize) -> PhysicalRegister {
    match param_num {
        0 => PhysicalRegister::_DI,
        1 => PhysicalRegister::_SI,
        2 => PhysicalRegister::_DX,
        3 => PhysicalRegister::_CX,
        4 => PhysicalRegister::R8,
        5 => PhysicalRegister::R9,
        6.. => panic!("this param should be on the stack. do it yourself")
    }
}
pub fn generate_return_value_reg(return_eightbyte_num: usize) -> PhysicalRegister {
    match return_eightbyte_num {
        0 => PhysicalRegister::_AX,
        1 => PhysicalRegister::_DX,
        2.. => panic!("this return value should be passed as a hidden pointer")
    }
}

impl RegisterName for LogicalRegister {
    fn generate_reg_name(&self, data_size: &MemoryLayout) -> String {
        let reg_as_physical = match self {
            LogicalRegister::ACC => PhysicalRegister::_AX,
            LogicalRegister::SECONDARY => PhysicalRegister::_CX,
            LogicalRegister::THIRD => PhysicalRegister::_DX,
        };

        return reg_as_physical.generate_reg_name(data_size);


    }
}
impl RegisterName for PhysicalRegister {
    fn generate_reg_name(&self, data_size: &MemoryLayout) -> String {
        match (self, data_size.size_bytes()) {
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

            _ => panic!("new register used, and this can't handle it")
        }.to_string()
    }
}