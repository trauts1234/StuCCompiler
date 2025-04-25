use crate::{asm_gen_data::{AsmData, GlobalAsmData}, assembly::assembly::Assembly, ast_metadata::ASTMetadata, block_statement::StatementOrDeclaration, compilation_state::functions::FunctionList, debugging::ASTDisplay, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, parse_data::ParseData};
use memory_size::MemorySize;

/**
 * this represents all the code inside a scope (i.e function definition)
 */
pub struct ScopeStatements {
    statements: Vec<StatementOrDeclaration>,
    local_scope_data: ParseData,//metadata to help with assembly generation
}

impl ScopeStatements {
    /**
     * tries to parse the tokens queue starting at previous_queue_idx, to find a scope, for a function or other
     * returns a ScopeStatements and the remaining tokens as a queue location, else none
     */
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, outer_scope_data: &ParseData) -> Option<ASTMetadata<ScopeStatements>> {
        let mut curr_queue_idx = previous_queue_idx.clone();

        let mut statements = Vec::new();
        //important! clone here so that variables and enums created in this scope do not leak out!
        let mut inner_scope_data = outer_scope_data.clone_for_new_scope();

        if Token::PUNCTUATOR(Punctuator::OPENSQUIGGLY) != tokens_queue.consume(&mut curr_queue_idx, &inner_scope_data)? {
            return None;//not enclosed in { }, so can't be a scope
        }

        let squiggly_close_idx = tokens_queue.find_matching_close_bracket(curr_queue_idx.index-1);//-1 since it has already been consumed
        
        //split to current tokens, and any after the slice
        let (mut curr_queue_idx, remaining_slice_after_scope) = tokens_queue.split_to_slices(squiggly_close_idx, &curr_queue_idx);

        //greedily consume as many statements as possible
        while let Some(ASTMetadata{resultant_tree, remaining_slice}) = StatementOrDeclaration::try_consume(tokens_queue, &curr_queue_idx, accessible_funcs, &mut inner_scope_data) {

            statements.push(resultant_tree);
            curr_queue_idx = remaining_slice;//jump to next one
        }

        //return the scope statements
        Some(ASTMetadata{
            resultant_tree: ScopeStatements {statements, local_scope_data: inner_scope_data}, 
            remaining_slice: remaining_slice_after_scope,
        })
    }

    pub fn generate_assembly(&self, asm_data: &AsmData, stack_data: &mut MemorySize, global_asm_data: &mut GlobalAsmData) -> Assembly {
        let mut result = Assembly::make_empty();

        let asm_data = asm_data.clone_for_new_scope(&self.local_scope_data, stack_data);

        for statement in &self.statements {
            let line_asm = statement.generate_assembly(&asm_data, stack_data, global_asm_data);
            result.merge(&line_asm);
        }

        result
    }
}

impl ASTDisplay for ScopeStatements {
    fn display_ast(&self, f: &mut crate::debugging::TreeDisplayInfo) {
        f.write("new scope");
        f.indent();
        for i in &self.statements {
            i.display_ast(f);
        }
        f.dedent();
    }
}