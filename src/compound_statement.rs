

use memory_size::MemoryLayout;

use crate::{ast_metadata::ASTMetadata, block_statement::StatementOrDeclaration, lexer::{token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size, stack_variables::StackVariables};
use std::fmt::Write;
/**
 * this represents all the code inside a scope (i.e function definition)
 */
#[derive(Debug)]
pub struct ScopeStatements {
    statements: Vec<StatementOrDeclaration>
}

impl ScopeStatements {
    /**
     * tries to parse the tokens queue starting at previous_queue_idx, to find a scope, for a function or other
     * returns a ScopeStatements and the remaining tokens as a queue location, else none
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, local_variables: &StackVariables) -> Option<ASTMetadata<ScopeStatements>> {
        let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        let mut statements = Vec::new();
        let mut all_scope_vars = local_variables.clone();
        let mut stack_used = MemoryLayout::new();

        if Token::PUNCTUATION("{".to_owned()) != tokens_queue.consume(&mut curr_queue_idx)? {
            return None;//not enclosed in { }, so can't be a scope
        }

        //greedily consume as many statements as possible
        while let Some(ASTMetadata{resultant_tree, remaining_slice, extra_stack_used}) = StatementOrDeclaration::try_consume(tokens_queue, &curr_queue_idx, &all_scope_vars) {
            statements.push(resultant_tree);
            curr_queue_idx = remaining_slice;//jump to next one
            stack_used += extra_stack_used;
        }

        if statements.len() == 0 {
            return None;
        }

        if Token::PUNCTUATION("}".to_owned()) != tokens_queue.consume(&mut curr_queue_idx)? {
            return None;//not enclosed in { }, so can't be a scope
        }

        //return the scope statements
        Some(ASTMetadata{
            resultant_tree: ScopeStatements {statements}, 
            remaining_slice: curr_queue_idx
        })
    }

    pub fn generate_assembly(&self) -> String {
        let mut result = String::new();

        for statement in &self.statements {
            write!(result, "{}", statement.generate_assembly()).unwrap();
        }

        result
    }
}