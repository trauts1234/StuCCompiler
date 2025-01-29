use crate::{compound_statement::ScopeStatements, expression::Expression};


pub enum Statement {
    //LABEL,//for goto, or switch cases
    EXPRESSION(Option<Expression>),
    COMPOUND(ScopeStatements),//this is a scope (not nescessarily for a function)
    //SELECTION,
    //ITERATION,
    //JUMP,
}