use crate::{data_type::recursive_data_type::DataType, debugging::ASTDisplay, expression::{expression::Expression, put_on_stack::PutOnStack}, expression_visitors::expr_visitor::ExprVisitor};

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

impl PutOnStack for CastExpression {
    fn put_on_stack(&self, asm_data: &crate::asm_gen_data::AsmData, stack: &mut stack_management::simple_stack_frame::SimpleStackFrame, global_asm_data: &crate::asm_gen_data::GlobalAsmData) -> (crate::assembly::assembly::Assembly, stack_management::stack_item::StackItemKey) {
        todo!()
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