use crate::{asm_gen_data::AsmData, asm_generation::{asm_comment, asm_line}, compilation_state::label_generator::LabelGenerator, data_type::{base_type::BaseType, data_type::DataType, type_modifier::DeclModifier}, expression::Expression, reference_assembly_visitor::ReferenceVisitor};
use std::fmt::Write;

#[derive(Debug, Clone, PartialEq)]
pub struct StringLiteral {
    text: Vec<i8>,//text plus zero terminator
    label: String
}

impl StringLiteral {
    pub fn generate_assembly(&self, asm_data: &AsmData) -> String {

        let mut visitor = ReferenceVisitor::new();

        Expression::STRINGLITERAL(self.clone()).accept(&mut visitor, asm_data);

        visitor.get_assembly()//decays to char*
    }

    pub fn get_num_chars(&self) -> usize {
        self.text.len()
    }
}

impl StringLiteral {
    pub fn get_label(&self) -> &str {
        &self.label
    }
    pub fn get_comma_separated_bytes(&self) -> String {
        self.text.iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(",")
    }
    pub fn try_new(inside_speechmarks: &str, string_label_generator: &mut LabelGenerator) -> Option<StringLiteral> {
        assert!(inside_speechmarks.is_ascii());
    
        Some(StringLiteral {
            label: format!("string_{}", string_label_generator.generate_label_number()),
            text: Self::use_escape_sequences(inside_speechmarks)
        })
    }

    pub fn use_escape_sequences(text: &str) -> Vec<i8> {
        text
        .replace("\\a", "\u{07}")//replace bell
        .replace("\\b", "\u{08}")//replace backspace
        .replace("\\f", "\u{0C}")//replace end page (form feed)
        .replace("\\n", "\n")//replace \n with an actual newline
        .replace("\\r", "\r")//replace \r
        .replace("\\t", "\t")//replace tab
        .replace("\\v", "\u{0B}")//replace vertical tab
        .replace("\\\\", "\\")//replace backslash (awkward because rust has to have escaped backslash too)
        .replace("\\'", "\'")//replace single quote
        .replace("\\\"", "\"")//replace double quote
        .replace("\\?", "?")//defend against trigraphs with \?
        //TODO hex escape sequences etc.
        .chars()
        .map(|x| x as i8)//convert to integers
        .chain(std::iter::once(0))//add null terminator 0
        .collect()
    }
}