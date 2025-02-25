use crate::memory_size::MemoryLayout;
/**
 * this funky macro runs like writeln!("text {}", foo), but
 * requires no unwrap and
 * automatically manages newlines
 */
macro_rules! asm_line {
    ($dest:expr, $($arg:tt)*) => {{
        let s = format!($($arg)*);
        if s.ends_with('\n') {
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
}

/**
 * name of an actual register
 */
#[derive(Clone, Copy)]
pub enum PhysicalRegister {
    _AX=0,
    _BX,
    _CX,
    _DX,
    _SI,
    _DI,
    _8=6,
    _9,

}

pub fn generate_param_reg(param_num: usize) -> PhysicalRegister {
    match param_num {
        0 => PhysicalRegister::_DI,
        1 => PhysicalRegister::_SI,
        2 => PhysicalRegister::_DX,
        3 => PhysicalRegister::_CX,
        4 => PhysicalRegister::_8,
        5 => PhysicalRegister::_9,
        6.. => panic!("this param should be on the stack. do it yourself")
    }
}

impl RegisterName for LogicalRegister {
    fn generate_reg_name(&self, data_size: &MemoryLayout) -> String {
        let prefix = match data_size.size_bytes() {
            4 => "e",
            8 => "r",
            _ => panic!("tried to create a register for {} bytes", data_size.size_bytes())
        }.to_string();

        let suffix = match self {
            LogicalRegister::ACC => "ax",
            LogicalRegister::SECONDARY => "cx",
        };

        return prefix + suffix;


    }
}
impl RegisterName for PhysicalRegister {
    fn generate_reg_name(&self, data_size: &MemoryLayout) -> String {
        const NEW_REGS_START: u8 = PhysicalRegister::_8 as u8;//r8,r9.. have special names for shorter versions of themselves

        match *self as u8 {
            ..NEW_REGS_START => {
                //handle old register types that are in the form (SIZELETTER)(REGNAME) i.e rax
                let reg_name = match self {
                    PhysicalRegister::_AX => "ax",
                    PhysicalRegister::_BX => "bx",
                    PhysicalRegister::_CX => "cx",
                    PhysicalRegister::_DX => "dx",
                    PhysicalRegister::_SI => "si",
                    PhysicalRegister::_DI => "di",
                    _ => panic!("new register used, and this can't handle it")
                };
                let prefix = match data_size.size_bytes() {
                    4 => "e",
                    8 => "r",
                    _ => panic!("tried to create a register for {} bytes", data_size.size_bytes())
                }.to_string();

                prefix + reg_name
            },
            NEW_REGS_START.. => {
                //handle new register types that are in the form r(REGNUMBER)(SIZELETTER)
                let reg_name = match self {
                    PhysicalRegister::_8 => "r8",
                    PhysicalRegister::_9 => "r9",
                    _ => panic!("old register used, and this can't handle it")
                }.to_string();

                let suffix = match data_size.size_bytes() {
                    4 => "d",
                    8 => "",
                    _ => panic!("tried to create a register for {} bytes", data_size.size_bytes())
                };

                reg_name + suffix
            }
        }
    }
}