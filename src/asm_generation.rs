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

pub fn generate_reg_name(data_size: &MemoryLayout, register_suffix: &str) -> String{
    let prefix = match data_size.size_bytes() {
        4 => "e",
        8 => "r",
        _ => panic!("tried to create a register for {} bytes with suffix {}", data_size.size_bytes(), register_suffix)
    };

    return prefix.to_string() + register_suffix;
}