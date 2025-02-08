

use memory_size::MemoryLayout;

use crate::{ast_metadata::ASTMetadata, block_statement::StatementOrDeclaration, label_generator::LabelGenerator, lexer::{token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size, stack_variables::StackVariables};
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
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, outer_variables: &StackVariables) -> Option<ASTMetadata<ScopeStatements>> {
        let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        let mut statements = Vec::new();
        let mut all_scope_vars = outer_variables.clone();

        if Token::PUNCTUATION("{".to_owned()) != tokens_queue.consume(&mut curr_queue_idx)? {
            return None;//not enclosed in { }, so can't be a scope
        }

        //greedily consume as many statements as possible
        while let Some(ASTMetadata{resultant_tree, remaining_slice, extra_stack_used}) = StatementOrDeclaration::try_consume(tokens_queue, &curr_queue_idx, &all_scope_vars) {
            if let StatementOrDeclaration::DECLARATION(decl) = &resultant_tree {
                assert!(extra_stack_used == decl.get_memory_usage());
                all_scope_vars.add_variable(decl.clone());//it was a variable, save it
            } else {
                assert!(extra_stack_used == MemoryLayout::new());//not creating a new variable, so no extra stack used
            }

            statements.push(resultant_tree);
            curr_queue_idx = remaining_slice;//jump to next one
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
            remaining_slice: curr_queue_idx,
            extra_stack_used: all_scope_vars.get_stack_used() - outer_variables.get_stack_used()//new variables - old variables
        })
    }

    pub fn generate_assembly(&self, label_gen: &mut LabelGenerator) -> String {
        let mut result = String::new();

        for statement in &self.statements {
            write!(result, "{}", statement.generate_assembly(label_gen)).unwrap();
        }

        result
    }
}