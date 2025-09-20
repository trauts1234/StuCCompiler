use crate::{data_type::recursive_data_type::DataType, debugging::ASTDisplay, expression::expression::Expression, expression_visitors::expr_visitor::ExprVisitor, generate_ir_traits::GetType};

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

impl GetType for CastExpression {
    fn get_type(&self, _asm_data: &crate::asm_gen_data::AsmData) -> DataType {
        self.new_type.clone()
    }
}

impl ASTDisplay for CastExpression {
    fn display_ast(&self, f: &mut crate::debugging::TreeDisplayInfo) {
        f.write(&format!("cast to {}", self.new_type));
        f.indent();
        self.expr.display_ast(f);
        f.dedent();
    }
}