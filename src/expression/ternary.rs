use crate::{data_type::recursive_data_type::DataType, expression::expression::Expression, expression_visitors::expr_visitor::ExprVisitor};

//TODO: a?b:c syntax parsing
//then generate assembly for it

#[derive(Clone, Debug)]
pub struct TernaryExpr {
    condition: Box<Expression>,
    true_branch: Box<Expression>,
    false_branch: Box<Expression>,
}

impl TernaryExpr {
    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_ternary(self)
    }

    pub fn get_data_type(&self) -> DataType {
        todo!()
    }
    pub fn true_branch(&self) -> &Expression {
        &self.true_branch
    }
    pub fn false_branch(&self) -> &Expression {
        &self.false_branch
    }
    pub fn condition(&self) -> &Expression {
        &self.condition
    }
}