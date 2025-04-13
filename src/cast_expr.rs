use crate::{data_type::recursive_data_type::DataType, expression::Expression, expression_visitors::expr_visitor::ExprVisitor};

#[derive(Clone)]
pub struct CastExpression {
    new_type: DataType,
    expr: Box<Expression>
}

impl CastExpression {

    pub fn new(new_type: DataType, expr: Expression) -> Self {
        Self {
            new_type,
            expr: Box::new(expr),
        }
    }

    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_cast_expr(self)
    }

    pub fn get_new_type(&self) -> &DataType {
        &self.new_type
    }
    pub fn get_uncasted_expr(&self) -> &Expression {
        &self.expr
    }
}