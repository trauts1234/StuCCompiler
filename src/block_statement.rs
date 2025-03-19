use crate::{asm_gen_data::AsmData, ast_metadata::ASTMetadata, compilation_state::{functions::FunctionList, label_generator::LabelGenerator}, declaration::InitialisedDeclaration, lexer::{token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size::MemoryLayout, parse_data::ParseData, statement::Statement};


/**
 * This represents either a statement or variable creation.
 * The sort of things found in functions
 */
pub enum StatementOrDeclaration {
    STATEMENT(Statement),
    DECLARATION(Vec<InitialisedDeclaration>),
}

impl StatementOrDeclaration {
    /**
     * tries to parse the tokens queue starting at previous_queue_idx, to find either a declaration or statement
     * returns a StatementOrDeclaration and the remaining tokens as a queue location, else none
     * local_variables must be mut, as declarations can modify this
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, scope_data: &mut ParseData) -> Option<ASTMetadata<StatementOrDeclaration>> {
        if previous_queue_idx.get_slice_size() == 0 {return None;}
        let curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        if let Some(ASTMetadata {remaining_slice, resultant_tree}) = Statement::try_consume(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data) {
            return Some(ASTMetadata{remaining_slice, resultant_tree: Self::STATEMENT(resultant_tree)});
        }

        if let Some(ASTMetadata {remaining_slice, resultant_tree}) = InitialisedDeclaration::try_consume(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data) {
            return Some(ASTMetadata{remaining_slice, resultant_tree: Self::DECLARATION(resultant_tree)});
        }

        None
    }

    pub fn generate_assembly(&self, label_gen: &mut LabelGenerator, asm_data: &AsmData) -> String {
        match self {
            Self::STATEMENT(statement) => statement.generate_assembly(label_gen, asm_data),
            Self::DECLARATION(decl) => {
                //declare each variable individually
                //no intermediate newline as generate_assembly puts in a trailing newline
                decl.iter().map(|x| x.generate_assembly(asm_data)).collect::<Vec<String>>().join("")
            },
        }
    }

    /**
     * if this is a declaration, I return None as I can't calculate the stack usage
     */
    pub fn get_stack_height(&self, asm_data: &AsmData) -> Option<MemoryLayout> {
        match self {
            StatementOrDeclaration::STATEMENT(statement) => statement.get_stack_height(asm_data),
            StatementOrDeclaration::DECLARATION(_) => None,
        }
    }
}