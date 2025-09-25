use std::fmt::Display;


#[derive(Default)]
pub struct RawAssembly {
    lines: Vec<String>
}

impl RawAssembly {
    pub fn add(&mut self, text: String) {
        assert!(!text.contains("\n"));
        self.lines.push(text);
    }
    pub fn add_comment<S: AsRef<str>>(&mut self, text: S) {
        assert!(!text.as_ref().contains("\n"));
        self.lines.push(format!("; {}", text.as_ref()));
    }
}

impl Display for RawAssembly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}