pub struct LabelGenerator {
    label_counter: u32,
}

impl LabelGenerator {
    pub fn new() -> Self {
        Self {
            label_counter: 0,
        }
    }

    pub fn generate_label(&mut self) -> String {
        let label = format!("label_{}", self.label_counter);
        self.label_counter += 1;
        label
    }
}