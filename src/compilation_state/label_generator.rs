#[derive(Debug, Default)]
pub struct LabelGenerator {
    label_counter: u32,
}

impl LabelGenerator {

    pub fn generate_label(&mut self) -> String {
        let label = format!("LABEL{}", self.label_counter);
        self.label_counter += 1;
        label
    }

    pub fn generate_label_number(&mut self) -> u32 {
        let result = self.label_counter;
        self.label_counter += 1;
        result
    }
}