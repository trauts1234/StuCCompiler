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

pub enum Register{
    ACC,
    SECONDARY,
}

pub fn generate_reg_name(data_size: &MemoryLayout, register: Register) -> String{
    let prefix = match data_size.size_bytes() {
        4 => "e",
        8 => "r",
        _ => panic!("tried to create a register for {} bytes", data_size.size_bytes())
    };

    let suffix = match register {
        Register::ACC => "ax",
        Register::SECONDARY => "cx",//next register that is NOT callee saved, so I can overwrite it
    };

    return prefix.to_string() + suffix;
}