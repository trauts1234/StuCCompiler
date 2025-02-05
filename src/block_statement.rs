
use crate::{ast_metadata::ASTMetadata, declaration::Declaration, lexer::{token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, stack_variables::StackVariables, statement::Statement};


/**
 * This represents either a statement or variable creation.
 * The sort of things found in functions
 */
#[derive(Debug)]
pub enum StatementOrDeclaration {
    STATEMENT(Statement),
    DECLARATION(Declaration)
}

impl StatementOrDeclaration {
    /**
     * tries to parse the tokens queue starting at previous_queue_idx, to find either a declaration or statement
     * returns a StatementOrDeclaration and the remaining tokens as a queue location, else none
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, local_variables: &StackVariables) -> Option<ASTMetadata<StatementOrDeclaration>> {
        let curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        if let Some(ASTMetadata {remaining_slice: remaining_tokens, resultant_tree: stat}) = Statement::try_consume(tokens_queue, &curr_queue_idx, local_variables) {
            return Some(ASTMetadata{remaining_slice: remaining_tokens, resultant_tree: Self::STATEMENT(stat)});
        }

        if let Some(ASTMetadata {remaining_slice: remaining_tokens, resultant_tree: decl}) = Declaration::try_consume(tokens_queue, &curr_queue_idx, local_variables) {
            return Some(ASTMetadata{remaining_slice: remaining_tokens, resultant_tree: Self::DECLARATION(decl)});
        }

        None
    }

    pub fn generate_assembly(&self) -> String {
        match self {
            Self::STATEMENT(statement) => statement.generate_assembly(),
            Self::DECLARATION(decl) => decl.generate_assembly(),
        }
    }
}