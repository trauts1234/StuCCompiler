use colored::Colorize;
use uuid::Uuid;
use unwrap_let::unwrap_let;
use crate::{assembly::{assembly::Assembly, comparison::AsmComparison, operand::{immediate::ImmediateValue, Operand}, operation::{AsmOperation, Label}}, data_type::{base_type::BaseType, recursive_data_type::{calculate_promoted_type_arithmetic, DataType}}, debugging::ASTDisplay, expression::{expression::Expression, put_on_stack::PutOnStack}, expression_visitors::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor, put_scalar_in_acc::ScalarInAccVisitor}};

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

impl PutOnStack for TernaryExpr {
    fn put_on_stack(&self, asm_data: &crate::asm_gen_data::AsmData, stack: &mut stack_management::simple_stack_frame::SimpleStackFrame, global_asm_data: &crate::asm_gen_data::GlobalAsmData) -> (crate::assembly::assembly::Assembly, stack_management::stack_item::StackItemKey) {
        let mut result = Assembly::make_empty();

        let generic_label = Uuid::new_v4().simple().to_string();
        let else_label = Label::Local(format!("{}_else", generic_label));//jump for the else branch
        let if_end_label = Label::Local(format!("{}_end", generic_label));//rendevous point for the if and else branches

        let true_type = self.true_branch.accept(&mut GetDataTypeVisitor {asm_data});
        let false_type = self.false_branch.accept(&mut GetDataTypeVisitor{asm_data});
        let promoted_type = calculate_promoted_type_arithmetic(&true_type, &false_type);

        let cond_false_label = &else_label;//only jump to else branch if it exists

        let condition_asm = self.condition.accept(&mut ScalarInAccVisitor {asm_data, stack_data: stack, global_asm_data});
        result.merge(&condition_asm);//generate the condition to acc
        
        unwrap_let!(DataType::RAW(BaseType::Scalar(condition_type)) = self.condition.accept(&mut GetDataTypeVisitor {asm_data}));

        //compare the result to 0
        result.add_instruction(AsmOperation::CMP {
            rhs: Operand::Imm(ImmediateValue("0".to_string())),
            data_type: condition_type
        });

        //if the result is 0, jump to the else block or the end of the if statement
        result.add_instruction(AsmOperation::JMPCC {
            label: cond_false_label.clone(),
            comparison: AsmComparison::EQ,
        });

        let (mut true_asm, true_location) = self.true_branch.put_on_stack(asm_data, stack, global_asm_data);
        true_asm.add_instruction(AsmOperation::CAST { from_type: true_type, to_type: promoted_type });
        let (mut false_asm, false_location) = self.false_branch.put_on_stack(asm_data, stack, global_asm_data);

        //true branch
        result.merge(&true_asm);
        //jump to the end of the if/else block
        result.add_instruction(AsmOperation::JMPCC {
            label: if_end_label.clone(),
            comparison: AsmComparison::ALWAYS,//unconditional jump
        });

        let else_body_asm = ternary.false_branch().accept(&mut ScalarInAccVisitor {asm_data: self.asm_data, stack_data: &mut self.stack_data, global_asm_data: self.global_asm_data});

        //start of the else block
        result.add_instruction(AsmOperation::Label(else_label));//add label
        result.merge(&else_body_asm);//generate the body of the else statement

        //after if/else are complete, jump here
        result.add_instruction(AsmOperation::Label(if_end_label));

        result
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