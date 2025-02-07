use memory_size::MemoryLayout;

use crate::{lexer::token_savepoint::TokenQueueSlice, memory_size};


/**
 * this stores return data for when nodes are generated for the AST
 */
pub struct ASTMetadata<NodeDataType> {
    pub(crate) remaining_slice: TokenQueueSlice,
    pub(crate) resultant_tree: NodeDataType,
    pub(crate) extra_stack_used: MemoryLayout,
}