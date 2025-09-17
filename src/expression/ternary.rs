use colored::Colorize;

use crate::{data_type::recursive_data_type::{calculate_promoted_type_arithmetic, DataType}, debugging::ASTDisplay, expression::expression::Expression, expression_visitors::{expr_visitor::ExprVisitor}, generate_ir::GetType};

//TODO: a?b:c syntax parsing
//then generate assembly for it

#[derive(Clone, Debug)]
pub struct TernaryExpr {
    condition: Box<Expression>,
    true_branch: Box<Expression>,
    false_branch: Box<Expression>,
}

impl TernaryExpr {
    pub fn new(condition: Expression, true_branch: Expression, false_branch: Expression) -> Self {
        Self {
            condition: Box::new(condition),
            true_branch: Box::new(true_branch),
            false_branch: Box::new(false_branch)
        }
    }
    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_ternary(self)
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

impl GetType for TernaryExpr {
    fn get_type(&self, asm_data: &crate::asm_gen_data::AsmData) -> DataType {
        calculate_promoted_type_arithmetic(
            &self.true_branch.accept(&mut GetDataTypeVisitor {asm_data}),
            &self.false_branch.accept(&mut GetDataTypeVisitor {asm_data})
        )
    }
}

impl ASTDisplay for TernaryExpr {
    fn display_ast(&self, f: &mut crate::debugging::TreeDisplayInfo) {
        f.write(&"ternary expression".red().to_string());
        f.indent();

        f.write(&"condition".green().to_string());
        f.indent();
        self.condition.display_ast(f);
        f.dedent();
        
        f.write(&"true branch".red().to_string());
        f.indent();
        self.true_branch.display_ast(f);
        f.dedent();

        f.write(&"false branch".red().to_string());
        f.indent();
        self.false_branch.display_ast(f);
        f.dedent();

        f.dedent();
    }
}