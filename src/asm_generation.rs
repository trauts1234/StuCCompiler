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