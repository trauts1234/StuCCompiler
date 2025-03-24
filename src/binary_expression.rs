
use crate::{asm_boilerplate::{self}, asm_gen_data::AsmData, asm_generation::{LogicalRegister, PhysicalRegister, RegisterName}, data_type::{base_type::BaseType, data_type::DataType}, expression::{generate_assembly_for_assignment, put_lhs_ax_rhs_cx, Expression}, expression_visitors::{data_type_visitor::GetDataTypeVisitor, put_scalar_in_acc::ScalarInAccVisitor}, lexer::punctuator::Punctuator, memory_size::MemoryLayout, number_literal::NumberLiteral};
use std::fmt::Write;
use crate::asm_generation::{asm_line, asm_comment};

#[derive(Clone)]
pub struct BinaryExpression {
    lhs: Box<Expression>,
    operator: Punctuator,
    rhs: Box<Expression>,
}

impl BinaryExpression {
    pub fn generate_assembly(&self, asm_data: &AsmData) -> String {
        let mut result = String::new();

        if self.operator == Punctuator::EQUALS {
            let lhs_type = self.lhs.accept(&mut GetDataTypeVisitor, asm_data);
            return generate_assembly_for_assignment(&self.lhs, &self.rhs, &lhs_type, &lhs_type.memory_size(), asm_data);
        }

        let promoted_type = match &self.operator {//I already have a function for this?
            Punctuator::EQUALS => self.lhs.accept(&mut GetDataTypeVisitor, asm_data),//assignment is just the lhs data size
            x if x.as_boolean_instr().is_some() => DataType::new_from_base_type(&BaseType::_BOOL, &Vec::new()),//is a boolean operator, operands are booleans
            _ => DataType::calculate_promoted_type_arithmetic(&self.lhs.accept(&mut GetDataTypeVisitor, asm_data), &self.rhs.accept(&mut GetDataTypeVisitor, asm_data))//else find a common meeting ground
        };
        let promoted_size = &promoted_type.memory_size();

        match &self.operator {
            Punctuator::PLUS => {
                asm_comment!(result, "adding {}-bit numbers", promoted_size.size_bits());


                asm_line!(result, "{}", self.lhs.accept(&mut ScalarInAccVisitor, asm_data));//put lhs in acc
                asm_line!(result, "{}", asm_boilerplate::cast_from_acc(&self.lhs.accept(&mut GetDataTypeVisitor, asm_data), &promoted_type));//cast to the correct type

                if self.rhs.accept(&mut GetDataTypeVisitor, asm_data).decay().is_pointer() {//adding array or pointer to int
                    //you can only add pointer and number here, as per the C standard

                    //get the size of rhs when it is dereferenced
                    let rhs_dereferenced_size_bytes = self.rhs.accept(&mut GetDataTypeVisitor, asm_data).remove_outer_modifier().memory_size().size_bytes();
                    //convert this number to a string
                    let rhs_deref_size_str = NumberLiteral::new(&rhs_dereferenced_size_bytes.to_string()).nasm_format();
                    asm_comment!(result, "rhs is a pointer. make lhs {} times bigger", rhs_deref_size_str);

                    assert!(promoted_type.memory_size().size_bytes() == 8);
                    asm_line!(result, "mov rcx, {}", rhs_deref_size_str);//get the size of value pointed to by rhs
                    asm_line!(result, "mul rcx");//multiply lhs by the size of value pointed to by rhs, so that +1 would skip along 1 value, not 1 byte
                    
                    //lhs is now in AX
                }

                //save lhs to stack, as preprocessing for it is done
                asm_line!(result, "{}", asm_boilerplate::push_reg(promoted_size, &LogicalRegister::ACC));

                asm_line!(result, "{}", self.rhs.accept(&mut ScalarInAccVisitor, asm_data));//put rhs in acc
                asm_line!(result, "{}", asm_boilerplate::cast_from_acc(&self.rhs.accept(&mut GetDataTypeVisitor, asm_data), &promoted_type));//cast to correct type

                if self.lhs.accept(&mut GetDataTypeVisitor, asm_data).decay().is_pointer() {
                    //you can only add pointer and number here, as per the C standard
                    //get the size of lhs when it is dereferenced
                    let lhs_dereferenced_size_bytes = self.lhs.accept(&mut GetDataTypeVisitor, asm_data).remove_outer_modifier().memory_size().size_bytes();
                    //convert this number to a string
                    let lhs_deref_size_str = NumberLiteral::new(&lhs_dereferenced_size_bytes.to_string()).nasm_format();

                    asm_comment!(result, "lhs is a pointer. make rhs {} times bigger", lhs_deref_size_str);

                    assert!(promoted_type.memory_size().size_bytes() == 8);
                    asm_line!(result, "mov rcx, {}", lhs_deref_size_str);//get the size of value pointed to by rhs
                    asm_line!(result, "mul rcx");//multiply rhs by the size of value pointed to by lhs, so that +1 would skip along 1 value, not 1 byte
                    
                    //rhs now in AX
                }

                //pop lhs to secondary register, since rhs is already in acc
                asm_line!(result, "{}", asm_boilerplate::pop_reg(&promoted_size, &LogicalRegister::SECONDARY));

                asm_line!(result, "add {}, {}",
                    LogicalRegister::ACC.generate_reg_name(promoted_size),
                    LogicalRegister::SECONDARY.generate_reg_name(promoted_size)
                );

                //result is now in AX
                
            },
            Punctuator::DASH => {
                asm_comment!(result, "subtracting numbers");
                asm_line!(result, "{}", put_lhs_ax_rhs_cx(&self.lhs, &self.rhs, &promoted_type, asm_data));

                asm_line!(result, "sub {}, {}",
                LogicalRegister::ACC.generate_reg_name(promoted_size),
                LogicalRegister::SECONDARY.generate_reg_name(promoted_size)
                );

            }
            Punctuator::ASTERISK => {
                asm_comment!(result, "multiplying numbers");
                asm_line!(result, "{}", put_lhs_ax_rhs_cx(&self.lhs, &self.rhs, &promoted_type, asm_data));

                assert!(promoted_type.underlying_type().is_signed());//unsigned multiply??
                assert!(promoted_type.underlying_type().is_integer());//floating point multiply??

                asm_line!(result, "imul {}, {}",
                    LogicalRegister::ACC.generate_reg_name(promoted_size),
                    LogicalRegister::SECONDARY.generate_reg_name(promoted_size)
                );

            },
            Punctuator::FORWARDSLASH => {
                asm_comment!(result, "dividing numbers");
                asm_line!(result, "{}", put_lhs_ax_rhs_cx(&self.lhs, &self.rhs, &promoted_type, asm_data));

                match (promoted_type.memory_size().size_bytes(), promoted_type.underlying_type().is_signed()) {
                    (4,true) => {
                        asm_line!(result, "{}", asm_boilerplate::I32_DIVIDE_AX_BY_CX);
                    },
                    (8,true) => {
                        asm_line!(result, "{}", asm_boilerplate::I64_DIVIDE_AX_BY_CX);
                    }
                    _ => panic!("unsupported operands for divide")
                }
            },

            Punctuator::PERCENT => {
                asm_comment!(result, "calculating modulus");
                asm_line!(result, "{}", put_lhs_ax_rhs_cx(&self.lhs, &self.rhs, &promoted_type, asm_data));

                //modulus is calculated using a DIV
                match (promoted_type.memory_size().size_bytes(), promoted_type.underlying_type().is_signed()) {
                    (4,true) => {
                        asm_line!(result, "{}", asm_boilerplate::I32_DIVIDE_AX_BY_CX);
                    },
                    (8,true) => {
                        asm_line!(result, "{}", asm_boilerplate::I64_DIVIDE_AX_BY_CX);
                    }
                    _ => panic!("unsupported operands")
                }

                //mod is returned in RDX
                asm_line!(result, "{}", asm_boilerplate::mov_reg(&promoted_size, &LogicalRegister::ACC,  &PhysicalRegister::_DX));
            }

            comparison if comparison.as_comparator_instr().is_some() => { // >, <, ==, >=, <=
                asm_comment!(result, "comparing numbers");
                asm_line!(result, "{}", put_lhs_ax_rhs_cx(&self.lhs, &self.rhs, &promoted_type, asm_data));

                let lhs_reg = LogicalRegister::ACC.generate_reg_name(promoted_size);
                let rhs_reg = LogicalRegister::SECONDARY.generate_reg_name(promoted_size);

                let result_size = MemoryLayout::from_bytes(1);
                let result_reg = LogicalRegister::ACC;

                asm_line!(result, "cmp {}, {}", lhs_reg, rhs_reg);//compare the two

                asm_line!(result, "{} {}", comparison.as_comparator_instr().unwrap(), result_reg.generate_reg_name(&result_size));//create the correct set instruction

            },

            operator if operator.as_boolean_instr().is_some() => {

                //perhaps this will work for binary operators too?
                asm_comment!(result, "applying boolean operator");

                asm_line!(result, "{}", put_lhs_ax_rhs_cx(&self.lhs, &self.rhs, &promoted_type, asm_data));

                assert!(promoted_size.size_bytes() == 1);//must be boolean
                assert!(promoted_type.underlying_type() == &BaseType::_BOOL);

                let lhs_reg = LogicalRegister::ACC.generate_reg_name(promoted_size);
                let rhs_reg = LogicalRegister::SECONDARY.generate_reg_name(promoted_size);

                let boolean_instruction = operator.as_boolean_instr().unwrap();
                
                asm_line!(result, "{} {}, {}", boolean_instruction, lhs_reg, rhs_reg);
            }

            _ => panic!("operator to binary expression is invalid")
        }

        result
    }

    pub fn get_data_type(&self, asm_data: &AsmData) -> DataType {
        match self.operator {
            Punctuator::PLUS |
            Punctuator::DASH |
            Punctuator::ASTERISK | 
            Punctuator::FORWARDSLASH | 
            Punctuator::PERCENT => DataType::calculate_promoted_type_arithmetic(&self.lhs.accept(&mut GetDataTypeVisitor, asm_data), &self.rhs.accept(&mut GetDataTypeVisitor, asm_data)),

            Punctuator::EQUALS => self.lhs.accept(&mut GetDataTypeVisitor, asm_data),//assigning, rhs must be converted to lhs

            Punctuator::ANGLELEFT |
            Punctuator::ANGLERIGHT |
            Punctuator::GREATEREQUAL |
            Punctuator::LESSEQUAL |
            Punctuator::DOUBLEEQUALS |
            Punctuator::PIPEPIPE |
            Punctuator::ANDAND |
            Punctuator::EXCLAMATIONEQUALS => DataType::new_from_base_type(&BaseType::_BOOL, &Vec::new()),

            _ => panic!("data type calculation for this binary operator is not implemented")
        }
    }
}

impl BinaryExpression {
    pub fn new(lhs: Expression, operator: Punctuator, rhs: Expression) -> BinaryExpression {
        BinaryExpression {
            lhs: Box::new(lhs),
            operator,
            rhs: Box::new(rhs),
        }
    }
}