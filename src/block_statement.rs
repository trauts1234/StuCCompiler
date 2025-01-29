use crate::{declaration::Declaration, statement::Statement};


/**
 * This represents either a statement or variable creation.
 * The sort of things found in functions
 */
pub enum BlockStatement {
    STATEMENT(Statement),
    DECLARATION(Declaration)
}