
use crate::{asm_boilerplate::{self}, asm_generation::{LogicalRegister, PhysicalRegister, RegisterName}, data_type::{base_type::BaseType, data_type::DataType}, expression::{generate_assembly_for_assignment, put_lhs_ax_rhs_cx, ExprNode}, lexer::punctuator::Punctuator, memory_size::MemoryLayout, number_literal::NumberLiteral};
use std::fmt::Write;
use crate::asm_generation::{asm_line, asm_comment};

pub struct BinaryExpression {
    lhs: Box<dyn ExprNode>,
    operator: Punctuator,
    rhs: Box<dyn ExprNode>,
}

impl ExprNode for BinaryExpression {
    fn generate_assembly(&self) -> String {
        let mut result = String::new();

        if self.operator == Punctuator::EQUALS {
            let lhs_type = self.lhs.get_data_type();
            return generate_assembly_for_assignment(&*self.lhs, &*self.rhs, &lhs_type, &lhs_type.memory_size());
        }

        let promoted_type = match self.operator {//I already have a function for this?
            Punctuator::EQUALS => self.lhs.get_data_type(),//assignment is just the lhs data size
            _ => DataType::calculate_promoted_type_arithmetic(&self.lhs.get_data_type(), &self.rhs.get_data_type())//else find a common meeting ground
        };
        let promoted_size = &promoted_type.memory_size();

        match &self.operator {
            Punctuator::PLUS => {
                asm_comment!(result, "adding {}-bit numbers", promoted_size.size_bits());

                asm_line!(result, "{}", self.lhs.generate_assembly());//put lhs in acc
                asm_line!(result, "{}", asm_boilerplate::cast_from_acc(&self.lhs.get_data_type(), &promoted_type));//cast to the correct type

                if self.rhs.get_data_type().is_pointer() {//adding pointer to int
                    //you can only add pointer and number here, as per the C standard

                    //get the size of rhs when it is dereferenced
                    let rhs_dereferenced_size_bytes = self.rhs.get_data_type().remove_outer_modifier().memory_size().size_bytes();
                    //convert this number to a string
                    let rhs_deref_size_str = NumberLiteral::try_new(&rhs_dereferenced_size_bytes.to_string()).unwrap().nasm_format();
                    asm_comment!(result, "rhs is a pointer. make lhs {} times bigger", rhs_deref_size_str);

                    assert!(promoted_type.memory_size().size_bytes() == 8);
                    asm_line!(result, "mov rcx, {}", rhs_deref_size_str);//get the size of value pointed to by rhs
                    asm_line!(result, "mul rcx");//multiply lhs by the size of value pointed to by rhs, so that +1 would skip along 1 value, not 1 byte
                    
                    //lhs is now in AX
                }

                //save lhs to stack, as preprocessing for it is done
                asm_line!(result, "{}", asm_boilerplate::push_reg(promoted_size, &LogicalRegister::ACC));

                asm_line!(result, "{}", self.rhs.generate_assembly());//put rhs in acc
                asm_line!(result, "{}", asm_boilerplate::cast_from_acc(&self.rhs.get_data_type(), &promoted_type));//cast to correct type

                if self.lhs.get_data_type().is_pointer() {
                    //you can only add pointer and number here, as per the C standard
                    //get the size of lhs when it is dereferenced
                    let lhs_dereferenced_size_bytes = self.lhs.get_data_type().remove_outer_modifier().memory_size().size_bytes();
                    //convert this number to a string
                    let lhs_deref_size_str = NumberLiteral::try_new(&lhs_dereferenced_size_bytes.to_string()).unwrap().nasm_format();

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
                asm_line!(result, "{}", put_lhs_ax_rhs_cx(&*self.lhs, &*self.rhs));

                asm_line!(result, "sub {}, {}",
                LogicalRegister::ACC.generate_reg_name(promoted_size),
                LogicalRegister::SECONDARY.generate_reg_name(promoted_size)
                );

            }
            Punctuator::ASTERISK => {
                asm_comment!(result, "multiplying numbers");
                asm_line!(result, "{}", put_lhs_ax_rhs_cx(&*self.lhs, &*self.rhs));

                assert!(promoted_type.underlying_type().is_signed());//unsigned multiply??
                assert!(promoted_type.underlying_type().is_integer());//floating point multiply??

                asm_line!(result, "imul {}, {}",
                    LogicalRegister::ACC.generate_reg_name(promoted_size),
                    LogicalRegister::SECONDARY.generate_reg_name(promoted_size)
                );

            },
            Punctuator::FORWARDSLASH => {
                asm_comment!(result, "dividing numbers");
                asm_line!(result, "{}", put_lhs_ax_rhs_cx(&*self.lhs, &*self.rhs));

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
                asm_line!(result, "{}", put_lhs_ax_rhs_cx(&*self.lhs, &*self.rhs));

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
                asm_line!(result, "{}", put_lhs_ax_rhs_cx(&*self.lhs, &*self.rhs));

                let lhs_reg = LogicalRegister::ACC.generate_reg_name(promoted_size);
                let rhs_reg = LogicalRegister::SECONDARY.generate_reg_name(promoted_size);

                let result_size = MemoryLayout::from_bytes(1);
                let result_reg = LogicalRegister::ACC;

                asm_line!(result, "cmp {}, {}", lhs_reg, rhs_reg);//compare the two

                asm_line!(result, "{} {}", comparison.as_comparator_instr().unwrap(), result_reg.generate_reg_name(&result_size));//create the correct set instruction

            },

            _ => panic!("operator to binary expression is invalid")
        }

        result
    }

    fn get_data_type(&self) -> DataType {
        match self.operator {
            Punctuator::PLUS |
            Punctuator::DASH |
            Punctuator::ASTERISK | 
            Punctuator::FORWARDSLASH | 
            Punctuator::PERCENT => DataType::calculate_promoted_type_arithmetic(&self.lhs.get_data_type(), &self.rhs.get_data_type()),

            Punctuator::EQUALS => self.lhs.get_data_type(),//assigning, rhs must be converted to lhs

            Punctuator::ANGLELEFT |
            Punctuator::ANGLERIGHT |
            Punctuator::GREATEREQUAL |
            Punctuator::LESSEQAUAL |
            Punctuator::DOUBLEEQUALS => DataType::new_from_base_type(&BaseType::_BOOL, &Vec::new()),

            _ => panic!("data type calculation for this binary operator is not implemented")
        }
    }

    fn put_lvalue_addr_in_acc(&self) -> String {
        todo!()
    }
}

impl BinaryExpression {
    pub fn new(lhs: Box<dyn ExprNode>, operator: Punctuator, rhs: Box<dyn ExprNode>) -> BinaryExpression {
        BinaryExpression {
            lhs,
            operator,
            rhs,
        }
    }
}