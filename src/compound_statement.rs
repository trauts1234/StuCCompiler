use crate::block_statement::BlockStatement;


/**
 * this represents all the code inside a scope (i.e function definition)
 */
pub struct ScopeStatements {
    statements: Vec<BlockStatement>
}