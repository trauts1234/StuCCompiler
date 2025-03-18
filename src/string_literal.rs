use crate::{asm_gen_data::AsmData, asm_generation::{asm_comment, asm_line}, compilation_state::label_generator::LabelGenerator, data_type::{base_type::BaseType, data_type::DataType, type_modifier::DeclModifier}, expression::ExprNode};
use std::fmt::Write;

#[derive(Debug, Clone, PartialEq)]
pub struct StringLiteral {
    text: Vec<i8>,//text plus zero terminator
    label: String
}

impl ExprNode for StringLiteral {
    fn generate_assembly(&self, asm_data: &AsmData) -> String {
        self.put_addr_in_acc(asm_data)//decays to char*
    }

    fn get_data_type(&self, _: &AsmData) -> DataType {
        //TODO maybe make it const char?
        DataType::new_from_base_type(&BaseType::I8, &vec![DeclModifier::ARRAY(self.text.len())])
    }
    
    fn put_addr_in_acc(&self, _: &AsmData) -> String {
        let mut result = String::new();
        asm_comment!(result, "getting address of string");
        asm_line!(result, "lea rax, [rel {}]", self.get_label());

        result
    }
    
    fn clone_self(&self) -> Box<dyn ExprNode> {
        Box::new(self.clone())
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