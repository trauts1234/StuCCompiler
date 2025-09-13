use stack_management::simple_stack_frame::SimpleStackFrame;

use crate::{asm_gen_data::{AsmData, GlobalAsmData}, assembly::assembly::Assembly, ast_metadata::ASTMetadata, compilation_state::label_generator::LabelGenerator, debugging::ASTDisplay, generate_ir::GenerateIR, initialised_declaration::InitialisedDeclaration, lexer::{token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, parse_data::ParseData, statement::Statement};

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
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData, struct_label_gen: &mut LabelGenerator) -> Option<ASTMetadata<StatementOrDeclaration>> {
        if previous_queue_idx.get_slice_size() == 0 {return None;}
        let curr_queue_idx = previous_queue_idx.clone();

        if let Some(ASTMetadata {remaining_slice, resultant_tree}) = Statement::try_consume(tokens_queue, &curr_queue_idx, scope_data, struct_label_gen) {
            return Some(ASTMetadata{remaining_slice, resultant_tree: Self::STATEMENT(resultant_tree)});
        }

        if let Some(ASTMetadata {remaining_slice, resultant_tree}) = InitialisedDeclaration::try_consume(tokens_queue, &curr_queue_idx,  scope_data, struct_label_gen) {
            return Some(ASTMetadata{remaining_slice, resultant_tree: Self::DECLARATION(resultant_tree)});
        }

        None
    }
}

impl GenerateIR for StatementOrDeclaration {
    fn generate_ir(&self, asm_data: &AsmData, stack_data: &mut SimpleStackFrame, global_asm_data: &GlobalAsmData) -> (Assembly, Option<stack_management::stack_item::StackItemKey>) {
        let asm = match self {
            Self::STATEMENT(statement) => statement.generate_assembly(asm_data, stack_data, global_asm_data),
            Self::DECLARATION(decl) => {
                //declare each variable individually
                //no intermediate newline as generate_assembly puts in a trailing newline
                decl
                .iter()
                .map(|x| x.generate_assembly(asm_data, stack_data, global_asm_data))//generate assembly
                .fold(Assembly::make_empty(), |mut acc, x| {
                    acc.merge(&x);
                    acc
                })
            },
        };

        (asm, None)
    }
}

impl ASTDisplay for StatementOrDeclaration {
    fn display_ast(&self, f: &mut crate::debugging::TreeDisplayInfo) {
        match self {
            StatementOrDeclaration::STATEMENT(statement) => statement.display_ast(f),
            StatementOrDeclaration::DECLARATION(initialised_declarations) =>  {
                for i in initialised_declarations {
                    i.display_ast(f);
                }
            }
        }
    }
}