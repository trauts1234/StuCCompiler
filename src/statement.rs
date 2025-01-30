use crate::{compound_statement::ScopeStatements, control_flow_statement::ControlFlowChange, expression::Expression, lexer::token_walk::TokenQueue};


pub enum Statement {
    //LABEL,//for goto, or switch cases
    EXPRESSION(Option<Expression>),
    COMPOUND(ScopeStatements),//this is a scope (not nescessarily for a function)
    //SELECTION,
    //ITERATION,
    CONTROLFLOW(ControlFlowChange),
}

impl Statement {
    pub fn try_consume_statement(tokens_queue: &mut TokenQueue) -> Option<Statement> {
        tokens_queue.save_checkpoint();

        if let Some(ss) = ScopeStatements::try_consume_statements(tokens_queue){
            tokens_queue.pop_checkpoint();
            return Some(Self::COMPOUND(ss));
        }

        if let Some(command) = tokens_queue;

        tokens_queue.pop_checkpoint();
        todo!()
    }
}