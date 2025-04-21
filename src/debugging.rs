use std::fmt::Write;

pub trait IRDisplay {
    fn display_ir(&self) -> String;
}

pub trait ASTDisplay {
    fn display_ast(&self, f: &mut TreeDisplayInfo);
}

pub trait DebugDisplay {
    fn display(&self) -> String;
}

#[derive(Default)]
pub struct TreeDisplayInfo {
    indent: usize,
    text: String
}

impl TreeDisplayInfo {
    pub fn write(&mut self, s: &str) {
        if s.is_empty() {
            writeln!(self.text, "{}", "|    ".repeat(self.indent)).unwrap();
            return;
        }
        assert!(!s.contains("\n"));
        let previous_nesting = "|    ".repeat(self.indent.saturating_sub(1));
        writeln!(self.text, "{}| -> {}", previous_nesting, s).unwrap();
    }

    pub fn indent(&mut self) {
        self.indent += 1;
        //self.write("");
    }
    pub fn dedent(&mut self) {
        self.indent -= 1;
        self.write("");
    }

    pub fn get_text(self) -> String {
        self.text
    }
}