use crate::{asm_generation::asm_line, ast_metadata::ASTMetadata, compilation_state::{functions::FunctionList, label_generator::LabelGenerator, stack_variables::StackVariables}, compound_statement::ScopeStatements, control_flow_statement::ControlFlowChange, expression::{self, ExprNode}, iteration_statement::IterationStatement, lexer::{token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, scope_data::ScopeData, selection_statement::SelectionStatement};
use std::fmt::Write;

pub enum Statement {
    EXPRESSION(Box<dyn ExprNode>),
    COMPOUND(ScopeStatements),//this is a scope (not nescessarily for a function)
    SELECTION(SelectionStatement),
    ITERATION(IterationStatement),
    CONTROLFLOW(ControlFlowChange),
}

impl Statement {
    /**
     * tries to parse the tokens queue starting at previous_queue_idx, to find a statement
     * returns a statement and the remaining tokens as a queue location, else none
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, scope_data: &mut ScopeData) -> Option<ASTMetadata<Statement>> {
        let curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        if let Some(ASTMetadata{resultant_tree, remaining_slice, extra_stack_used}) = ScopeStatements::try_consume(tokens_queue, &curr_queue_idx, accessible_funcs, &scope_data){
            return Some(ASTMetadata{resultant_tree: Self::COMPOUND(resultant_tree), remaining_slice, extra_stack_used});
        }

        if let Some(ASTMetadata{resultant_tree, remaining_slice, extra_stack_used}) = SelectionStatement::try_consume(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data){
            return Some(ASTMetadata{resultant_tree: Self::SELECTION(resultant_tree), remaining_slice, extra_stack_used});
        }

        if let Some(ASTMetadata{resultant_tree, remaining_slice, extra_stack_used}) = IterationStatement::try_consume(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data){
            return Some(ASTMetadata{resultant_tree: Self::ITERATION(resultant_tree), remaining_slice, extra_stack_used});
        }

        if let Some(ASTMetadata{resultant_tree, remaining_slice, extra_stack_used}) = ControlFlowChange::try_consume(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data){
            return Some(ASTMetadata{resultant_tree: Self::CONTROLFLOW(resultant_tree), remaining_slice, extra_stack_used});
        }

        if let Some(ASTMetadata{resultant_tree, remaining_slice, extra_stack_used}) = expression::try_consume(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data){
            return Some(ASTMetadata{resultant_tree: Self::EXPRESSION(resultant_tree), remaining_slice, extra_stack_used});
        }

        None
    }

    pub fn generate_assembly(&self, label_gen: &mut LabelGenerator) -> String {
        let mut result = String::new();

        match self {
            Self::COMPOUND(scope) => {
                asm_line!(result, "{}", scope.generate_assembly(label_gen));
            }
            Self::CONTROLFLOW(command) => {
                asm_line!(result, "{}", command.generate_assembly());
            }
            Self::EXPRESSION(expr) => {
                asm_line!(result, "{}", expr.generate_assembly());
            }
            Self::SELECTION(selection) => {
                asm_line!(result, "{}", selection.generate_assembly(label_gen));
            },
            Self::ITERATION(it) => {
                asm_line!(result, "{}", it.generate_assembly(label_gen));
            }
        }

        return result;
    }
}