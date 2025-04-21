use crate::{data_type::recursive_data_type::DataType, debugging::{ASTDisplay, DebugDisplay}, expression::expression::Expression, expression_visitors::expr_visitor::ExprVisitor};

#[derive(Clone, Debug)]
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

impl ASTDisplay for CastExpression {
    fn display_ast(&self, f: &mut crate::debugging::TreeDisplayInfo) {
        f.write(&format!("cast to {}", self.new_type.display()));
        f.indent();
        self.expr.display_ast(f);
        f.dedent();
    }
}