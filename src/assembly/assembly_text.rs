use std::fmt::Display;


#[derive(Default)]
pub struct RawAssembly {
    lines: Vec<String>
}

impl RawAssembly {
    pub fn add(&mut self, text: String) {
        assert!(!text.ends_with("\n"));
        self.lines.push(text);
    }
    pub fn add_comment<S: AsRef<str>>(&mut self, text: S) {
        assert!(!text.as_ref().contains("\n"));
        self.lines.push(format!("; {}", text.as_ref()));
    }
}

impl Display for RawAssembly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for l in &self.lines {
            writeln!(f, "{}", l)?;
        }

        Ok(())
    }
}