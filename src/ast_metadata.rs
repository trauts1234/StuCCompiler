use crate::lexer::token_savepoint::TokenQueueSlice;


/**
 * this stores return data for when nodes are generated for the AST
 */
pub struct ASTMetadata<NodeDataType> {
    pub(crate) remaining_slice: TokenQueueSlice,
    pub(crate) resultant_tree: NodeDataType,
}