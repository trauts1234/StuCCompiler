use crate::{asm_generation::asm_line, ast_metadata::ASTMetadata, block_statement::StatementOrDeclaration, compilation_state::{functions::FunctionList, label_generator::LabelGenerator, stack_variables::StackVariables}, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}};
use std::fmt::Write;
/**
 * this represents all the code inside a scope (i.e function definition)
 * TODO clone scope variables, as this has its own scope
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
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, outer_variables: &StackVariables, accessible_funcs: &FunctionList) -> Option<ASTMetadata<ScopeStatements>> {
        let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        let mut statements = Vec::new();
        let mut all_scope_vars = outer_variables.clone();

        if Token::PUNCTUATOR(Punctuator::OPENSQUIGGLY) != tokens_queue.consume(&mut curr_queue_idx)? {
            return None;//not enclosed in { }, so can't be a scope
        }

        //greedily consume as many statements as possible
        while let Some(ASTMetadata{resultant_tree, remaining_slice, extra_stack_used: _}) = StatementOrDeclaration::try_consume(tokens_queue, &curr_queue_idx, &mut all_scope_vars, accessible_funcs) {

            statements.push(resultant_tree);
            curr_queue_idx = remaining_slice;//jump to next one
        }

        if Token::PUNCTUATOR(Punctuator::CLOSESQUIGGLY) != tokens_queue.consume(&mut curr_queue_idx)? {
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
            asm_line!(result, "{}", statement.generate_assembly(label_gen));
        }

        result
    }
}