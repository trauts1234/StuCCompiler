
/**
 * This represents either a statement or variable creation.
 * The sort of things found in functions
 */
enum BlockStatement {
    STATEMENT,
    DECLARATION(Declaration)
}