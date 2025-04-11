use crate::{compilation_state::label_generator::LabelGenerator, expression_visitors::expr_visitor::ExprVisitor};

#[derive(Debug, Clone, PartialEq)]
pub struct StringLiteral {
    text: Vec<i8>,//text plus zero terminator
    label: String
}

impl StringLiteral {

    pub fn get_num_chars(&self) -> usize {
        self.text.len()
    }

    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_string_literal(self)
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
        .replace("\\0", "\0")//replace end of string
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