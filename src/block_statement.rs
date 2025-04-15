use crate::{asm_gen_data::AsmData, assembly::assembly::Assembly, ast_metadata::ASTMetadata, compilation_state::{functions::FunctionList, label_generator::LabelGenerator}, initialised_declaration::InitialisedDeclaration, lexer::{token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, parse_data::ParseData, statement::Statement};
use memory_size::MemorySize;

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
        let curr_queue_idx = previous_queue_idx.clone();

        if let Some(ASTMetadata {remaining_slice, resultant_tree}) = Statement::try_consume(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data) {
            return Some(ASTMetadata{remaining_slice, resultant_tree: Self::STATEMENT(resultant_tree)});
        }

        if let Some(ASTMetadata {remaining_slice, resultant_tree}) = InitialisedDeclaration::try_consume(tokens_queue, &curr_queue_idx, accessible_funcs, scope_data) {
            return Some(ASTMetadata{remaining_slice, resultant_tree: Self::DECLARATION(resultant_tree)});
        }

        None
    }

    pub fn generate_assembly(&self, label_gen: &mut LabelGenerator, asm_data: &AsmData, stack_data: &mut MemorySize) -> Assembly {
        match self {
            Self::STATEMENT(statement) => statement.generate_assembly(label_gen, asm_data, stack_data),
            Self::DECLARATION(decl) => {
                //declare each variable individually
                //no intermediate newline as generate_assembly puts in a trailing newline
                decl
                .iter()
                .map(|x| x.generate_assembly(asm_data, stack_data))//generate assembly
                .fold(Assembly::make_empty(), |mut acc, x| {
                    acc.merge(&x);
                    acc
                })
            },
        }
    }
}