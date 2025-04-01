use crate::{asm_gen_data::AsmData, asm_generation::asm_line, assembly_metadata::AssemblyMetadata, ast_metadata::ASTMetadata, compilation_state::{functions::FunctionList, label_generator::LabelGenerator, stack_used::StackUsage}, compound_statement::ScopeStatements, control_flow_statement::ControlFlowChange, expression::{self, Expression}, expression_visitors::put_scalar_in_acc::ScalarInAccVisitor, iteration_statement::IterationStatement, lexer::{token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size::MemoryLayout, parse_data::ParseData, selection_statement::SelectionStatement};
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
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, scope_data: &ParseData) -> Option<ASTMetadata<Statement>> {
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

    pub fn generate_assembly(&self, label_gen: &mut LabelGenerator, asm_data: &AsmData, stack_data: &mut StackUsage) -> AssemblyMetadata {

        //match on variant and call recursively
        match self {
            Self::COMPOUND(scope) => {
                scope.generate_assembly(label_gen, asm_data, stack_data)
            }
            Self::CONTROLFLOW(command) => {
                command.generate_assembly(asm_data)
            }
            Self::EXPRESSION(expr) => {
                expr.accept(&mut ScalarInAccVisitor {asm_data})
            }
            Self::SELECTION(selection) => {
                selection.generate_assembly(label_gen, asm_data, stack_data)
            },
            Self::ITERATION(it) => {
                it.generate_assembly(label_gen, asm_data, &stack_data)
            }
        }
    }

    pub fn get_stack_height(&self, asm_data: &AsmData, stack_data: StackUsage) -> Option<MemoryLayout> {
        panic!("this is already done in generate_assembly!");
        match self {
            Statement::EXPRESSION(_) => None,//calculations take no stack long-term
            Statement::COMPOUND(scope_statements) => Some(scope_statements.get_stack_height(asm_data, stack_data)),//scope contains useful metadata
            Statement::SELECTION(selection_statement) => selection_statement.get_stack_height(asm_data),//finds whichever uses the most stack: if or else
            Statement::ITERATION(iteration_statement) => Some(iteration_statement.get_stack_height(asm_data)),//takes into account iterator variables
            Statement::CONTROLFLOW(_) => None,//return statements take no stack long-term
        }
    }
}