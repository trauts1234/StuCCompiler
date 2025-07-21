use colored::Colorize;
use unwrap_let::unwrap_let;

use crate::{assembly::{assembly::Assembly, comparison::AsmComparison, operation::AsmOperation}, ast_metadata::ASTMetadata, debugging::ASTDisplay, lexer::{keywords::Keyword, punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, parse_data::ParseData};

/// A label in the style `label:`
/// 
/// This is not currently suited for switch cases `case 1+2:` `default:`
pub struct CustomLabel(String);

impl CustomLabel {
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<ASTMetadata<Self>> {
        let mut curr_queue_idx = previous_queue_idx.clone();

        if let Some(Token::IDENTIFIER(label)) = tokens_queue.consume(&mut curr_queue_idx, scope_data) {
            if tokens_queue.consume(&mut curr_queue_idx, scope_data) == Some(Token::PUNCTUATOR(Punctuator::COLON)) {
                Some(ASTMetadata { remaining_slice: curr_queue_idx, resultant_tree: CustomLabel(label) })
            } else {
                None//label, but not label:
            }
        } else {
            None //no label name
        }
    }

    pub fn generate_assembly(&self) -> Assembly {
        let mut result = Assembly::make_empty();

        result.add_commented_instruction(AsmOperation::Label { name: self.0.clone() }, format!("custom label {}", self.0));

        result
    }
}

pub struct Goto(String);

impl Goto {
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<ASTMetadata<Self>> {
        let mut curr_queue_idx = previous_queue_idx.clone();
        //ensure this is a "goto x"
        if tokens_queue.consume(&mut curr_queue_idx, &scope_data) != Some(Token::KEYWORD(Keyword::GOTO)) {return None;}

        unwrap_let!(Some(Token::IDENTIFIER(label_name)) = tokens_queue.consume(&mut curr_queue_idx, &scope_data));

        Some(ASTMetadata { remaining_slice: curr_queue_idx, resultant_tree: Goto(label_name) })
    }

    pub fn generate_assembly(&self) -> Assembly {
        let mut result = Assembly::make_empty();

        result.add_commented_instruction(
            AsmOperation::JMPCC { label: self.0.clone(), comparison: AsmComparison::ALWAYS },
            format!("goto {}", self.0)
        );

        result
    }
}

impl ASTDisplay for CustomLabel {
    fn display_ast(&self, f: &mut crate::debugging::TreeDisplayInfo) {
        f.write(&format!("custom label {}", self.0.yellow()));
    }
}

impl ASTDisplay for Goto {
    fn display_ast(&self, f: &mut crate::debugging::TreeDisplayInfo) {
        f.write(&format!("goto {}", self.0.yellow()))
    }
}