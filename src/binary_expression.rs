
use unwrap_let::unwrap_let;

use crate::{asm_gen_data::AsmData, assembly::assembly::Assembly, data_type::{base_type::BaseType, recursive_data_type::{calculate_promoted_type_arithmetic, RecursiveDataType}}, expression::{generate_assembly_for_assignment, Expression}, expression_visitors::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor}, lexer::punctuator::Punctuator, memory_size::MemoryLayout, number_literal::NumberLiteral};

#[derive(Clone)]
pub struct BinaryExpression {
    lhs: Box<Expression>,
    operator: Punctuator,
    rhs: Box<Expression>,
}

impl BinaryExpression {
    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_binary_expression(self)
    }
    
    pub fn generate_assembly(&self, asm_data: &AsmData, stack_data: &mut MemoryLayout) -> Assembly {
        let mut result = Assembly::make_empty();

        if self.operator == Punctuator::EQUALS {
            return generate_assembly_for_assignment(&self.lhs, &self.rhs, asm_data, stack_data);
        }

        let lhs_type = self.lhs.accept(&mut GetDataTypeVisitor {asm_data});
        let rhs_type = self.rhs.accept(&mut GetDataTypeVisitor {asm_data});

        let promoted_type = match &self.operator {//I already have a function for this?
            Punctuator::EQUALS => panic!("assignment already done"),
            x if x.as_boolean_instr().is_some() => RecursiveDataType::RAW(BaseType::_BOOL),//is a boolean operator, operands are booleans
            _ => calculate_promoted_type_arithmetic(&lhs_type, &rhs_type)//else find a common meeting ground
        };
        let promoted_size = promoted_type.memory_size(asm_data);

        match &self.operator {
            Punctuator::PLUS => {
                asm_comment!(result, "adding {}-bit numbers", promoted_size.size_bits());


                asm_line!(result, "{}", self.lhs.accept(&mut ScalarInAccVisitor {asm_data, stack_data}));//put lhs in acc
                asm_line!(result, "{}", asm_boilerplate::cast_from_acc(&lhs_type, &promoted_type, asm_data));//cast to the correct type

                if let RecursiveDataType::POINTER(_) = self.rhs.accept(&mut GetDataTypeVisitor {asm_data}).decay() {//adding array or pointer to int
                    //you can only add pointer and number here, as per the C standard

                    //get the size of rhs when it is dereferenced
                    let rhs_dereferenced_size_bytes = rhs_type.remove_outer_modifier().memory_size(asm_data).size_bytes();
                    //convert this number to a string
                    let rhs_deref_size_str = NumberLiteral::new(&rhs_dereferenced_size_bytes.to_string()).nasm_format();
                    asm_comment!(result, "rhs is a pointer. make lhs {} times bigger", rhs_deref_size_str);

                    assert!(promoted_type.memory_size(asm_data).size_bytes() == 8);
                    asm_line!(result, "mov rcx, {}", rhs_deref_size_str);//get the size of value pointed to by rhs
                    asm_line!(result, "mul rcx");//multiply lhs by the size of value pointed to by rhs, so that +1 would skip along 1 value, not 1 byte
                    
                    //lhs is now in AX
                }

                //save lhs to stack, as preprocessing for it is done
                *stack_data += promoted_size;//allocate temporary lhs storage
                let lhs_temporary_address = stack_data.clone();
                asm_line!(result, "{}", asm_boilerplate::mov_asm(promoted_size, &RAMLocation::SubFromBP(lhs_temporary_address), &LogicalRegister::ACC));

                asm_line!(result, "{}", self.rhs.accept(&mut ScalarInAccVisitor {asm_data, stack_data}));//put rhs in acc
                asm_line!(result, "{}", asm_boilerplate::cast_from_acc(&rhs_type, &promoted_type, asm_data));//cast to correct type

                if let RecursiveDataType::POINTER(_) = self.lhs.accept(&mut GetDataTypeVisitor {asm_data}).decay() {
                    //you can only add pointer and number here, as per the C standard
                    //get the size of lhs when it is dereferenced
                    let lhs_dereferenced_size_bytes = lhs_type.remove_outer_modifier().memory_size(asm_data).size_bytes();
                    //convert this number to a string
                    let lhs_deref_size_str = NumberLiteral::new(&lhs_dereferenced_size_bytes.to_string()).nasm_format();

                    asm_comment!(result, "lhs is a pointer. make rhs {} times bigger", lhs_deref_size_str);

                    assert!(promoted_type.memory_size(asm_data).size_bytes() == 8);
                    asm_line!(result, "mov rcx, {}", lhs_deref_size_str);//get the size of value pointed to by rhs
                    asm_line!(result, "mul rcx");//multiply rhs by the size of value pointed to by lhs, so that +1 would skip along 1 value, not 1 byte
                    
                    //rhs now in AX
                }

                //read lhs to secondary register, since rhs is already in acc
                asm_line!(result, "{}", asm_boilerplate::mov_asm(promoted_size, &LogicalRegister::SECONDARY, &RAMLocation::SubFromBP(lhs_temporary_address)));

                asm_line!(result, "add {}, {}",
                    LogicalRegister::ACC.generate_name(promoted_size),
                    LogicalRegister::SECONDARY.generate_name(promoted_size)
                );

                //result is now in AX
                
            },
            Punctuator::DASH => {
                asm_comment!(result, "subtracting numbers");
                asm_line!(result, "{}", put_lhs_ax_rhs_cx(&self.lhs, &self.rhs, &promoted_type, asm_data, stack_data));

                asm_line!(result, "sub {}, {}",
                LogicalRegister::ACC.generate_name(promoted_size),
                LogicalRegister::SECONDARY.generate_name(promoted_size)
                );

            }
            Punctuator::ASTERISK => {
                asm_comment!(result, "multiplying numbers");
                asm_line!(result, "{}", put_lhs_ax_rhs_cx(&self.lhs, &self.rhs, &promoted_type, asm_data, stack_data));

                unwrap_let!(RecursiveDataType::RAW(promoted_underlying) = promoted_type);
                assert!(promoted_underlying.is_integer() && promoted_underlying.is_signed());//unsigned multiply?? floating point multiply??

                asm_line!(result, "imul {}, {}",
                    LogicalRegister::ACC.generate_name(promoted_size),
                    LogicalRegister::SECONDARY.generate_name(promoted_size)
                );

            },
            Punctuator::FORWARDSLASH => {
                asm_comment!(result, "dividing numbers");
                asm_line!(result, "{}", put_lhs_ax_rhs_cx(&self.lhs, &self.rhs, &promoted_type, asm_data, stack_data));

                unwrap_let!(RecursiveDataType::RAW(promoted_underlying) = promoted_type);
                assert!(promoted_underlying.is_integer());//floating point division??

                match (promoted_underlying.memory_size(asm_data).size_bytes(), promoted_underlying.is_signed()) {
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
                asm_line!(result, "{}", put_lhs_ax_rhs_cx(&self.lhs, &self.rhs, &promoted_type, asm_data, stack_data));

                unwrap_let!(RecursiveDataType::RAW(promoted_underlying) = promoted_type);
                assert!(promoted_underlying.is_integer());//floating point division??

                //modulus is calculated using a DIV
                match (promoted_underlying.memory_size(asm_data).size_bytes(), promoted_underlying.is_signed()) {
                    (4,true) => {
                        asm_line!(result, "{}", asm_boilerplate::I32_DIVIDE_AX_BY_CX);
                    },
                    (8,true) => {
                        asm_line!(result, "{}", asm_boilerplate::I64_DIVIDE_AX_BY_CX);
                    }
                    _ => panic!("unsupported operands")
                }

                //mod is returned in RDX
                asm_line!(result, "{}", asm_boilerplate::mov_asm(promoted_size, &LogicalRegister::ACC,  &PhysicalRegister::_DX));
            }

            comparison if comparison.as_comparator_instr().is_some() => { // >, <, ==, >=, <=
                asm_comment!(result, "comparing numbers");
                asm_line!(result, "{}", put_lhs_ax_rhs_cx(&self.lhs, &self.rhs, &promoted_type, asm_data, stack_data));

                let lhs_reg = LogicalRegister::ACC.generate_name(promoted_size);
                let rhs_reg = LogicalRegister::SECONDARY.generate_name(promoted_size);

                let result_size = MemoryLayout::from_bytes(1);
                let result_reg = LogicalRegister::ACC;

                asm_line!(result, "cmp {}, {}", lhs_reg, rhs_reg);//compare the two

                asm_line!(result, "{} {}", comparison.as_comparator_instr().unwrap(), result_reg.generate_name(result_size));//create the correct set instruction

            },

            operator if operator.as_boolean_instr().is_some() => {

                //perhaps this will work for binary operators too?
                //warning: what if either side is not a boolean
                asm_comment!(result, "applying boolean operator");

                asm_line!(result, "{}", put_lhs_ax_rhs_cx(&self.lhs, &self.rhs, &promoted_type, asm_data, stack_data));

                unwrap_let!(RecursiveDataType::RAW(promoted_underlying) = promoted_type);
                assert!(promoted_underlying.is_integer());//floating point division??

                assert!(promoted_underlying.memory_size(asm_data).size_bytes() == 1);//must be boolean
                assert!(promoted_underlying == BaseType::_BOOL);

                let lhs_reg = LogicalRegister::ACC.generate_name(promoted_size);
                let rhs_reg = LogicalRegister::SECONDARY.generate_name(promoted_size);

                let boolean_instruction = operator.as_boolean_instr().unwrap();
                
                asm_line!(result, "{} {}, {}", boolean_instruction, lhs_reg, rhs_reg);
            }

            _ => panic!("operator to binary expression is invalid")
        }

        result
    }

    pub fn get_data_type(&self, asm_data: &AsmData) -> RecursiveDataType {
        match self.operator {
            Punctuator::PLUS |
            Punctuator::DASH |
            Punctuator::ASTERISK | 
            Punctuator::FORWARDSLASH | 
            Punctuator::PERCENT => {
                calculate_promoted_type_arithmetic(//calculate type when data types:
                    &self.lhs.accept(&mut GetDataTypeVisitor { asm_data }),//type of lhs
                    &self.rhs.accept(&mut GetDataTypeVisitor { asm_data }),//type of rhs
                )
            },

            Punctuator::EQUALS => self.lhs.accept(&mut GetDataTypeVisitor {asm_data}),//assigning, rhs must be converted to lhs

            Punctuator::ANGLELEFT |
            Punctuator::ANGLERIGHT |
            Punctuator::GREATEREQUAL |
            Punctuator::LESSEQUAL |
            Punctuator::DOUBLEEQUALS |
            Punctuator::PIPEPIPE |
            Punctuator::ANDAND |
            Punctuator::EXCLAMATIONEQUALS => RecursiveDataType::RAW(BaseType::_BOOL),

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