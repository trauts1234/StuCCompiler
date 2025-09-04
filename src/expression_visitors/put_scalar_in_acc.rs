use crate::{asm_boilerplate::cast_from_acc, asm_gen_data::{AsmData, GetStructUnion, GlobalAsmData}, assembly::{assembly::Assembly, comparison::AsmComparison, operand::{immediate::{ImmediateValue, ToImmediate}, memory_operand::MemoryOperand, register::GPRegister, Operand, RegOrMem}, operation::{AsmOperation, Label}}, data_type::{base_type::{BaseType, IntegerType, ScalarType}, recursive_data_type::DataType}, expression::unary_prefix_expr::UnaryPrefixExpression, expression_visitors::{put_struct_on_stack::CopyStructVisitor, reference_assembly_visitor::ReferenceVisitor}, stack_allocation::StackAllocator, member_access::MemberAccess};
use unwrap_let::unwrap_let;
use super::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor};


/**
 * calculates the value of the expression and puts the scalar result in AX. does not leave anything on the stack
 * does not work with structs, as they are not scalar types
 * stack_data is modified
 */
pub struct ScalarInAccVisitor<'a>{
    pub(crate) asm_data: &'a AsmData,
    pub(crate) stack_data: &'a mut StackAllocator,
    pub(crate) global_asm_data: &'a mut GlobalAsmData
}

impl<'a> ExprVisitor for ScalarInAccVisitor<'a> {
    type Output = Assembly;

    fn visit_number_literal(&mut self, number: &crate::number_literal::typed_value::NumberLiteral) -> Self::Output {
        let mut result = Assembly::make_empty();

        let reg_size = number.get_data_type().memory_size();//decide how much storage is needed to temporarily store the constant
        result.add_comment(format!("reading number literal: {:?}", number));

        result.add_instruction(AsmOperation::MOV {
            to: RegOrMem::GPReg(GPRegister::acc()),
            from: Operand::Imm(number.as_imm()),
            size: reg_size,
        });

        result
    }

    fn visit_variable(&mut self, var: &crate::declaration::MinimalDataVariable) -> Self::Output {
        let mut result = Assembly::make_empty();

        let my_type = var.accept(&mut GetDataTypeVisitor{asm_data: self.asm_data});

        if let DataType::ARRAY {..} = my_type {//is array
            //getting an array, decays to a pointer
            result.add_comment(format!("decaying array {} to pointer", var.name));
            let addr_asm = var.accept(&mut ReferenceVisitor{asm_data: self.asm_data, stack_data: self.stack_data, global_asm_data: self.global_asm_data});
            result.merge(&addr_asm);

        } else {
            let reg_size = my_type.memory_size(self.asm_data);//decide which register size is appropriate for this variable
            result.add_comment(format!("reading variable: {}", var.name));

            result.add_instruction(AsmOperation::MOV {
                to: RegOrMem::GPReg(GPRegister::acc()),
                from: Operand::Mem(self.asm_data.get_variable(&var.name).location.clone()),
                size: reg_size,
            });
        }

        result
    }

    fn visit_string_literal(&mut self, string: &crate::string_literal::StringLiteral) -> Self::Output {
        let mut result = Assembly::make_empty();
        result.add_instruction(AsmOperation::LEA {
            from: MemoryOperand::LabelAccess(string.get_label().to_string())
        });//warning: duplicated code from get address

        result
    }

    fn visit_func_call(&mut self, func_call: &crate::function_call::FunctionCall) -> Self::Output {
        func_call.generate_assembly_scalar_return(self)
    }

    fn visit_unary_prefix(&mut self, expr: &UnaryPrefixExpression) -> Self::Output {
        expr.generate_assembly(self.asm_data, self.stack_data, self.global_asm_data)
    }

    fn visit_unary_postfix(&mut self, expr: &crate::expression::unary_postfix_expression::UnaryPostfixExpression) -> Self::Output {
        expr.generate_assembly(self.asm_data, self.stack_data, self.global_asm_data)
    }

    fn visit_binary_expression(&mut self, expr: &crate::binary_expression::BinaryExpression) -> Self::Output {
        expr.generate_assembly(self.asm_data, self.stack_data, self.global_asm_data)
    }

    fn visit_member_access(&mut self, member_access: &MemberAccess) -> Self::Output {
        let mut result = Assembly::make_empty();

        let member_name = member_access.get_member_name();
        unwrap_let!(DataType::RAW(BaseType::Struct(original_struct_name)) = member_access.get_base_tree().accept(&mut GetDataTypeVisitor{asm_data: self.asm_data}));

        let original_struct_definition = self.asm_data.get_struct(&original_struct_name);
        let (member_decl, member_offset) = original_struct_definition.get_member_data(member_name);

        //allocate struct on the stack
        let resultant_struct_location = self.stack_data.allocate(original_struct_definition.calculate_size().unwrap());
        let resultant_struct_location = Operand::Mem(MemoryOperand::SubFromBP(resultant_struct_location));

        result.add_comment(format!("getting struct's member {}", member_name));

        if let DataType::ARRAY { .. } = member_decl.data_type {
            //get pointer to struct
            let struct_addr_asm = member_access.get_base_tree().accept(&mut ReferenceVisitor{asm_data:self.asm_data, stack_data: self.stack_data, global_asm_data: self.global_asm_data});
            result.merge(&struct_addr_asm);

            //offset the start pointer to the address of the member
            result.add_instruction(AsmOperation::ADD {
                increment: Operand::Imm(member_offset.as_imm()),
                data_type: ScalarType::Integer(IntegerType::U64),
            });
        } else {
            //clone the struct, just in case it is a return value for example
            let struct_clone_asm = member_access.get_base_tree().accept(&mut CopyStructVisitor{asm_data: self.asm_data, stack_data: self.stack_data, global_asm_data: self.global_asm_data, resultant_location: resultant_struct_location});
            result.merge(&struct_clone_asm);//generate struct that I am getting member of

            //offset the start pointer to the address of the member
            result.add_instruction(AsmOperation::ADD {
                increment: Operand::Imm(member_offset.as_imm()),
                data_type: ScalarType::Integer(IntegerType::U64),
            });

            //dereference pointer
            result.add_instruction(AsmOperation::MOV {
                to: RegOrMem::GPReg(GPRegister::acc()),
                from: Operand::Mem(MemoryOperand::MemoryAddress{pointer_reg: GPRegister::acc() }),
                size: member_decl.data_type.memory_size(self.asm_data),
            });
        }

        //asm_line!(result, "{}", member_access.accept(&mut PopStructFromStack{asm_data: self.asm_data}));//pop the struct if needed

        result
    }
    
    fn visit_cast_expr(&mut self, expr: &crate::cast_expr::CastExpression) -> Self::Output {
        let mut result = Assembly::make_empty();
        let uncasted_asm = expr.get_uncasted_expr().accept(self);
        let uncasted_type = expr.get_uncasted_expr().accept(&mut GetDataTypeVisitor{asm_data:self.asm_data});
        //generate uncasted data
        result.merge(&uncasted_asm);
        //cast to required type
        result.merge(&cast_from_acc(&uncasted_type, expr.get_new_type(), self.asm_data));

        result
    }
    
    fn visit_sizeof(&mut self, sizeof: &crate::expression::sizeof_expression::SizeofExpr) -> Self::Output {
        sizeof.generate_assembly(self.asm_data)
    }
    
    fn visit_ternary(&mut self, ternary: &crate::expression::ternary::TernaryExpr) -> Self::Output {
        let mut result = Assembly::make_empty();

        let generic_label = self.global_asm_data.label_gen_mut().generate_label();
        let else_label = Label::Local(format!("{}_else", generic_label));//jump for the else branch
        let if_end_label = Label::Local(format!("{}_end", generic_label));//rendevous point for the if and else branches

        let cond_false_label = &else_label;//only jump to else branch if it exists

        let condition_asm = ternary.condition().accept(&mut ScalarInAccVisitor {asm_data: self.asm_data, stack_data: self.stack_data, global_asm_data: self.global_asm_data});
        result.merge(&condition_asm);//generate the condition to acc
        
        unwrap_let!(DataType::RAW(BaseType::Scalar(condition_type)) = ternary.condition().accept(&mut GetDataTypeVisitor {asm_data: self.asm_data}));

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

        //both branches are mutually exclusive so I only need enough stack for *one* of the branches to run
        let (mut if_body_stack_usage, mut else_body_stack_usage) = self.stack_data.split_for_branching();

        //generate the body of the if statement
        let if_body_asm = ternary.true_branch().accept(&mut ScalarInAccVisitor {asm_data: self.asm_data, stack_data: &mut if_body_stack_usage, global_asm_data: self.global_asm_data});
        result.merge(&if_body_asm);

        //jump to the end of the if/else block
        result.add_instruction(AsmOperation::JMPCC {
            label: if_end_label.clone(),
            comparison: AsmComparison::ALWAYS,//unconditional jump
        });

        let else_body_asm = ternary.false_branch().accept(&mut ScalarInAccVisitor {asm_data: self.asm_data, stack_data: &mut else_body_stack_usage, global_asm_data: self.global_asm_data});

        //start of the else block
        result.add_instruction(AsmOperation::Label(else_label));//add label
        result.merge(&else_body_asm);//generate the body of the else statement

        //stack required is the largest between the if and else branches
        self.stack_data.merge_from_branching(if_body_stack_usage, else_body_stack_usage);

        //after if/else are complete, jump here
        result.add_instruction(AsmOperation::Label(if_end_label));

        result

    }
}