use crate::{asm_gen_data::{AsmData, GlobalAsmData}, assembly::assembly::Assembly, ast_metadata::ASTMetadata, compilation_state::label_generator::LabelGenerator, compound_statement::ScopeStatements, control_flow_statement::ControlFlowChange, debugging::ASTDisplay, expression::expression::Expression, expression_visitors::put_scalar_in_acc::ScalarInAccVisitor, iteration_statement::IterationStatement, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, parse_data::ParseData, selection_statement::SelectionStatement};
use memory_size::MemorySize;

pub enum Statement {
    EXPRESSION(Expression),
    COMPOUND(ScopeStatements),//this is a scope (not nescessarily for a function)
    SELECTION(SelectionStatement),
    ITERATION(IterationStatement),
    CONTROLFLOW(ControlFlowChange),
    NOP,//for example, the line of code ";;;;;;;;"
}

impl Statement {
    /**
     * tries to parse the tokens queue starting at previous_queue_idx, to find a statement
     * returns a statement and the remaining tokens as a queue location, else none
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData, struct_label_gen: &mut LabelGenerator) -> Option<ASTMetadata<Statement>> {
        let curr_queue_idx = previous_queue_idx.clone();

        if let Some(ASTMetadata{resultant_tree, remaining_slice}) = ScopeStatements::try_consume(tokens_queue, &curr_queue_idx, &scope_data, struct_label_gen){
            return Some(ASTMetadata{resultant_tree: Self::COMPOUND(resultant_tree), remaining_slice});
        }

        if let Some(ASTMetadata{resultant_tree, remaining_slice}) = SelectionStatement::try_consume(tokens_queue, &curr_queue_idx, scope_data, struct_label_gen){
            return Some(ASTMetadata{resultant_tree: Self::SELECTION(resultant_tree), remaining_slice});
        }

        if let Some(ASTMetadata{resultant_tree, remaining_slice}) = IterationStatement::try_consume(tokens_queue, &curr_queue_idx, scope_data, struct_label_gen){
            return Some(ASTMetadata{resultant_tree: Self::ITERATION(resultant_tree), remaining_slice});
        }

        if let Some(ASTMetadata{resultant_tree, remaining_slice}) = ControlFlowChange::try_consume(tokens_queue, &curr_queue_idx, scope_data, struct_label_gen){
            return Some(ASTMetadata{resultant_tree: Self::CONTROLFLOW(resultant_tree), remaining_slice});
        }

        if let Some(ASTMetadata{resultant_tree, remaining_slice}) = Expression::try_consume(tokens_queue, &curr_queue_idx, scope_data, struct_label_gen){
            return Some(ASTMetadata{resultant_tree: Self::EXPRESSION(resultant_tree), remaining_slice});
        }

        if tokens_queue.peek(&curr_queue_idx, scope_data) == Some(Token::PUNCTUATOR(Punctuator::SEMICOLON)) {
            //just a ; so is a nop
            return Some(ASTMetadata { remaining_slice: curr_queue_idx.next_clone(), resultant_tree: Self::NOP });
        }

        None
    }

    pub fn generate_assembly(&self, asm_data: &AsmData, stack_data: &mut MemorySize, global_asm_data: &mut GlobalAsmData) -> Assembly {

        //match on variant and call recursively
        match self {
            Self::COMPOUND(scope) => {
                        scope.generate_assembly(asm_data, stack_data, global_asm_data)
                    }
            Self::CONTROLFLOW(command) => {
                        command.generate_assembly(asm_data, stack_data)
                    }
            Self::EXPRESSION(expr) => {
                        expr.accept(&mut ScalarInAccVisitor {asm_data, stack_data})
                    }
            Self::SELECTION(selection) => {
                        selection.generate_assembly(asm_data, stack_data, global_asm_data)
                    },
            Self::ITERATION(it) => {
                        it.generate_assembly(asm_data, stack_data, global_asm_data)
                    }

            Self::NOP => Assembly::make_empty(),
        }
    }
}

impl ASTDisplay for Statement {
    fn display_ast(&self, f: &mut crate::debugging::TreeDisplayInfo) {
        match self {
            Statement::EXPRESSION(expression) => expression.display_ast(f),
            Statement::COMPOUND(scope_statements) => scope_statements.display_ast(f),
            Statement::SELECTION(selection_statement) => selection_statement.display_ast(f),
            Statement::ITERATION(iteration_statement) => iteration_statement.display_ast(f),
            Statement::CONTROLFLOW(control_flow_change) => control_flow_change.display_ast(f),
            Statement::NOP => f.write("NOP"),
        }
    }
}