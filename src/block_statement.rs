
use crate::{ast_metadata::ASTMetadata, compilation_state::{functions::FunctionList, label_generator::LabelGenerator, stack_variables::StackVariables}, declaration::InitialisedDeclaration, lexer::{token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, statement::Statement};


/**
 * This represents either a statement or variable creation.
 * The sort of things found in functions
 */
#[derive(Debug)]
pub enum StatementOrDeclaration {
    STATEMENT(Statement),
    DECLARATION(Vec<InitialisedDeclaration>)
}

impl StatementOrDeclaration {
    /**
     * tries to parse the tokens queue starting at previous_queue_idx, to find either a declaration or statement
     * returns a StatementOrDeclaration and the remaining tokens as a queue location, else none
     * local_variables must be mut, as declarations can modify this
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, local_variables: &mut StackVariables, accessible_funcs: &FunctionList) -> Option<ASTMetadata<StatementOrDeclaration>> {
        if previous_queue_idx.get_slice_size() == 0 {return None;}
        let curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        if let Some(ASTMetadata {remaining_slice, resultant_tree, extra_stack_used}) = Statement::try_consume(tokens_queue, &curr_queue_idx, local_variables, accessible_funcs) {
            return Some(ASTMetadata{remaining_slice, resultant_tree: Self::STATEMENT(resultant_tree), extra_stack_used});
        }

        if let Some(ASTMetadata {remaining_slice, resultant_tree, extra_stack_used}) = InitialisedDeclaration::try_consume(tokens_queue, &curr_queue_idx, local_variables, accessible_funcs) {
            return Some(ASTMetadata{remaining_slice, resultant_tree: Self::DECLARATION(resultant_tree), extra_stack_used});
        }

        None
    }

    pub fn generate_assembly(&self, label_gen: &mut LabelGenerator) -> String {
        match self {
            Self::STATEMENT(statement) => statement.generate_assembly(label_gen),
            Self::DECLARATION(decl) => {
                //declare each variable individually
                //no intermediate newline as generate_assembly puts in a trailing newline
                decl.iter().map(|x| x.generate_assembly()).collect::<Vec<String>>().join("")
            },
        }
    }
}