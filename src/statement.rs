use crate::{asm_gen_data::AsmData, asm_generation::asm_line, ast_metadata::ASTMetadata, compilation_state::{functions::FunctionList, label_generator::LabelGenerator}, compound_statement::ScopeStatements, control_flow_statement::ControlFlowChange, expression::{self, Expression}, iteration_statement::IterationStatement, lexer::{token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size::MemoryLayout, parse_data::ParseData, selection_statement::SelectionStatement};
use std::fmt::Write;

pub enum Statement {
    EXPRESSION(Expression),
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
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, scope_data: &mut ParseData) -> Option<ASTMetadata<Statement>> {
        let curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        if let Some(ASTMetadata{resultant_tree, remaining_slice}) = ScopeStatements::try_consume(tokens_queue, &curr_queue_idx, accessible_funcs, &scope_data){
            return Some(ASTMetadata{resultant_tree: Self::COMPOUND(resultant_tree), remaining_slice});
        }

        if let Some(ASTMetadata{resultant_tree, remaining_slice}) = SelectionStatement::try_consume(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data){
            return Some(ASTMetadata{resultant_tree: Self::SELECTION(resultant_tree), remaining_slice});
        }

        if let Some(ASTMetadata{resultant_tree, remaining_slice}) = IterationStatement::try_consume(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data){
            return Some(ASTMetadata{resultant_tree: Self::ITERATION(resultant_tree), remaining_slice});
        }

        if let Some(ASTMetadata{resultant_tree, remaining_slice}) = ControlFlowChange::try_consume(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data){
            return Some(ASTMetadata{resultant_tree: Self::CONTROLFLOW(resultant_tree), remaining_slice});
        }

        if let Some(ASTMetadata{resultant_tree, remaining_slice}) = expression::try_consume(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data){
            return Some(ASTMetadata{resultant_tree: Self::EXPRESSION(resultant_tree), remaining_slice});
        }

        None
    }

    pub fn generate_assembly(&self, label_gen: &mut LabelGenerator, asm_data: &AsmData) -> String {
        let mut result = String::new();

        match self {
            Self::COMPOUND(scope) => {
                asm_line!(result, "{}", scope.generate_assembly(label_gen, asm_data));
            }
            Self::CONTROLFLOW(command) => {
                asm_line!(result, "{}", command.generate_assembly(asm_data));
            }
            Self::EXPRESSION(expr) => {
                asm_line!(result, "{}", expr.generate_assembly(asm_data));
            }
            Self::SELECTION(selection) => {
                asm_line!(result, "{}", selection.generate_assembly(label_gen, asm_data));
            },
            Self::ITERATION(it) => {
                asm_line!(result, "{}", it.generate_assembly(label_gen, asm_data));
            }
        }

        return result;
    }

    pub fn get_stack_height(&self, asm_data: &AsmData) -> Option<MemoryLayout> {
        match self {
            Statement::EXPRESSION(_) => None,//calculations take no stack long-term
            Statement::COMPOUND(scope_statements) => Some(scope_statements.get_stack_height(asm_data)),//scope contains useful metadata
            Statement::SELECTION(selection_statement) => selection_statement.get_stack_height(asm_data),//finds whichever uses the most stack: if or else
            Statement::ITERATION(iteration_statement) => Some(iteration_statement.get_stack_height(asm_data)),//takes into account iterator variables
            Statement::CONTROLFLOW(_) => None,//return statements take no stack long-term
        }
    }
}