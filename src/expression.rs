use crate::{l_value::LValue, r_value::RValue, token::operator::Operator};


pub enum Expression {
    RVALUE(RValue),
    ASSIGNMENT(LValue, Operator, Box<Expression>)// a = b;
}