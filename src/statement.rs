
enum Statement {
    LABELED,
    EXPRESSION,
    COMPOUND(CompoundStatement),
    SELECTION,
    ITERATION,
    JUMP,
}