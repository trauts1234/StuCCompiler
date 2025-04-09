
use unwrap_let::unwrap_let;

use crate::{asm_boilerplate::cast_from_acc, asm_gen_data::AsmData, assembly::{assembly::Assembly, operand::{Operand, AsmRegister}, operation::AsmOperation}, data_type::{base_type::BaseType, recursive_data_type::{calculate_promoted_type_arithmetic, DataType}}, expression::{generate_assembly_for_assignment, put_lhs_ax_rhs_cx, Expression}, expression_visitors::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor, put_scalar_in_acc::ScalarInAccVisitor}, lexer::punctuator::Punctuator, memory_size::MemoryLayout, number_literal::NumberLiteral};

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
            x if x.as_boolean_instr().is_some() => DataType::RAW(BaseType::_BOOL),//is a boolean operator, operands are booleans
            _ => calculate_promoted_type_arithmetic(&lhs_type, &rhs_type)//else find a common meeting ground
        };
        let promoted_size = promoted_type.memory_size(asm_data);

        match &self.operator {
            Punctuator::PLUS => {
                result.add_comment(format!("adding {}-bit numbers", promoted_size.size_bits()));

                let lhs_asm = self.lhs.accept(&mut ScalarInAccVisitor {asm_data, stack_data});
                let lhs_cast_asm = cast_from_acc(&lhs_type, &promoted_type, asm_data);
                result.merge(&lhs_asm);//put lhs in acc
                result.merge(&lhs_cast_asm);//cast to the correct type

                if let DataType::POINTER(_) = self.rhs.accept(&mut GetDataTypeVisitor {asm_data}).decay() {//adding array or pointer to int
                    //you can only add pointer and number here, as per the C standard

                    //get the size of rhs when it is dereferenced
                    let rhs_dereferenced_size_bytes = rhs_type.remove_outer_modifier().memory_size(asm_data).size_bytes();
                    //convert this number to a string
                    let rhs_deref_size_str = NumberLiteral::new(&rhs_dereferenced_size_bytes.to_string()).nasm_format();
                    result.add_comment(format!("rhs is a pointer. make lhs {} times bigger", rhs_deref_size_str));

                    assert!(promoted_type.memory_size(asm_data).size_bytes() == 8);
                    //get the size of value pointed to by rhs
                    result.add_instruction(AsmOperation::MOV {
                        to: Operand::Register(AsmRegister::_CX),
                        from: Operand::ImmediateValue(rhs_deref_size_str),
                        size: MemoryLayout::from_bytes(8),
                    });

                    //multiply lhs by the size of value pointed to by rhs, so that +1 would skip along 1 value, not 1 byte
                    result.add_instruction(AsmOperation::MUL {
                        multiplier: Operand::Register(AsmRegister::_CX),
                        data_type: promoted_type.clone(),
                    });
                    
                    //lhs is now in AX
                }

                //save lhs to stack, as preprocessing for it is done
                *stack_data += promoted_size;//allocate temporary lhs storage
                let lhs_temporary_address = stack_data.clone();
                result.add_instruction(AsmOperation::MOV {
                    to: Operand::SubFromBP(lhs_temporary_address),
                    from: Operand::Register(AsmRegister::acc()),
                    size: promoted_size
                });

                //calculate and cast rhs value
                let rhs_asm = self.rhs.accept(&mut ScalarInAccVisitor {asm_data, stack_data});
                let rhs_cast_asm = cast_from_acc(&rhs_type, &promoted_type, asm_data);
                result.merge(&rhs_asm);
                result.merge(&rhs_cast_asm);


                if let DataType::POINTER(_) = self.lhs.accept(&mut GetDataTypeVisitor {asm_data}).decay() {
                    //you can only add pointer and number here, as per the C standard
                    //get the size of lhs when it is dereferenced
                    let lhs_dereferenced_size_bytes = lhs_type.remove_outer_modifier().memory_size(asm_data).size_bytes();
                    //convert this number to a string
                    let lhs_deref_size_str = NumberLiteral::new(&lhs_dereferenced_size_bytes.to_string()).nasm_format();

                    result.add_comment(format!("lhs is a pointer. make rhs {} times bigger", lhs_deref_size_str));

                    assert!(promoted_type.memory_size(asm_data).size_bytes() == 8);
                    //get the size of value pointed to by rhs
                    result.add_instruction(AsmOperation::MOV {
                        to: Operand::Register(AsmRegister::_CX),
                        from: Operand::ImmediateValue(lhs_deref_size_str),
                        size: MemoryLayout::from_bytes(8),
                    });

                    //multiply lhs by the size of value pointed to by rhs, so that +1 would skip along 1 value, not 1 byte
                    result.add_instruction(AsmOperation::MUL {
                        multiplier: Operand::Register(AsmRegister::_CX),
                        data_type: promoted_type.clone(),
                    });

                    //rhs now in AX
                }

                //read lhs to secondary register, since rhs is already in acc
                result.add_instruction(AsmOperation::MOV { to: Operand::Register(AsmRegister::secondary()), from: Operand::SubFromBP(lhs_temporary_address), size: promoted_size });

                result.add_instruction(AsmOperation::ADD {
                    destination: Operand::Register(AsmRegister::acc()),
                    increment: Operand::Register(AsmRegister::secondary()),
                    data_type: promoted_type
                });

                //result is now in AX
                
            },
            Punctuator::DASH => {
                result.add_comment(format!("subtracting {}-bit numbers", promoted_size.size_bits()));

                let lhs_asm = self.lhs.accept(&mut ScalarInAccVisitor {asm_data, stack_data});
                let lhs_cast_asm = cast_from_acc(&lhs_type, &promoted_type, asm_data);
                result.merge(&lhs_asm);//put lhs in acc
                result.merge(&lhs_cast_asm);//cast to the correct type

                if let DataType::POINTER(_) = self.rhs.accept(&mut GetDataTypeVisitor {asm_data}).decay() {//subtractinging array or pointer to int
                    //you can only subtract pointer and number here, as per the C standard

                    //get the size of rhs when it is dereferenced
                    let rhs_dereferenced_size_bytes = rhs_type.remove_outer_modifier().memory_size(asm_data).size_bytes();
                    //convert this number to a string
                    let rhs_deref_size_str = NumberLiteral::new(&rhs_dereferenced_size_bytes.to_string()).nasm_format();
                    result.add_comment(format!("rhs is a pointer. make lhs {} times bigger", rhs_deref_size_str));

                    assert!(promoted_type.memory_size(asm_data).size_bytes() == 8);
                    //get the size of value pointed to by rhs
                    result.add_instruction(AsmOperation::MOV {
                        to: Operand::Register(AsmRegister::_CX),
                        from: Operand::ImmediateValue(rhs_deref_size_str),
                        size: MemoryLayout::from_bytes(8),
                    });

                    //multiply lhs by the size of value pointed to by rhs, so that +1 would skip along 1 value, not 1 byte
                    result.add_instruction(AsmOperation::MUL {
                        multiplier: Operand::Register(AsmRegister::_CX),
                        data_type: promoted_type.clone(),
                    });
                    
                    //lhs is now in AX
                }

                //save lhs to stack, as preprocessing for it is done
                *stack_data += promoted_size;//allocate temporary lhs storage
                let lhs_temporary_address = stack_data.clone();
                result.add_instruction(AsmOperation::MOV {
                    to: Operand::SubFromBP(lhs_temporary_address),
                    from: Operand::Register(AsmRegister::acc()),
                    size: promoted_size
                });

                //calculate and cast rhs value
                let rhs_asm = self.rhs.accept(&mut ScalarInAccVisitor {asm_data, stack_data});
                let rhs_cast_asm = cast_from_acc(&rhs_type, &promoted_type, asm_data);
                result.merge(&rhs_asm);
                result.merge(&rhs_cast_asm);


                if let DataType::POINTER(_) = self.lhs.accept(&mut GetDataTypeVisitor {asm_data}).decay() {
                    //you can only add pointer and number here, as per the C standard
                    //get the size of lhs when it is dereferenced
                    let lhs_dereferenced_size_bytes = lhs_type.remove_outer_modifier().memory_size(asm_data).size_bytes();
                    //convert this number to a string
                    let lhs_deref_size_str = NumberLiteral::new(&lhs_dereferenced_size_bytes.to_string()).nasm_format();

                    result.add_comment(format!("lhs is a pointer. make rhs {} times bigger", lhs_deref_size_str));

                    assert!(promoted_type.memory_size(asm_data).size_bytes() == 8);
                    //get the size of value pointed to by rhs
                    result.add_instruction(AsmOperation::MOV {
                        to: Operand::Register(AsmRegister::_CX),
                        from: Operand::ImmediateValue(lhs_deref_size_str),
                        size: MemoryLayout::from_bytes(8),
                    });

                    //multiply lhs by the size of value pointed to by rhs, so that +1 would skip along 1 value, not 1 byte
                    result.add_instruction(AsmOperation::MUL {
                        multiplier: Operand::Register(AsmRegister::_CX),
                        data_type: promoted_type.clone(),
                    });

                    //rhs now in AX
                }

                //put RHS in CX 
                result.add_instruction(AsmOperation::MOV { to: Operand::Register(AsmRegister::secondary()), from: Operand::Register(AsmRegister::acc()), size: MemoryLayout::from_bytes(8)});

                //read lhs to acc
                result.add_instruction(AsmOperation::MOV { to: Operand::Register(AsmRegister::acc()), from: Operand::SubFromBP(lhs_temporary_address), size: promoted_size });

                result.add_instruction(AsmOperation::SUB {
                    destination: Operand::Register(AsmRegister::acc()),
                    decrement: Operand::Register(AsmRegister::secondary()),
                    data_type: promoted_type
                });

                //result is now in AX

            }
            Punctuator::ASTERISK => {
                result.add_comment("mulitplying numbers");

                result.merge(&put_lhs_ax_rhs_cx(&self.lhs, &self.rhs, &promoted_type, asm_data, stack_data));

                unwrap_let!(DataType::RAW(promoted_underlying) = &promoted_type);
                assert!(promoted_underlying.is_integer() && promoted_underlying.is_signed());//unsigned multiply?? floating point multiply??

                result.add_instruction(AsmOperation::MUL {
                    multiplier: Operand::Register(AsmRegister::secondary()),
                    data_type: promoted_type
                });

            },
            Punctuator::FORWARDSLASH => {
                result.add_comment("dividing numbers");

                result.merge(&put_lhs_ax_rhs_cx(&self.lhs, &self.rhs, &promoted_type, asm_data, stack_data));

                unwrap_let!(DataType::RAW(_) = promoted_type);

                result.add_instruction(AsmOperation::DIV {
                    divisor: Operand::Register(AsmRegister::secondary()),
                    data_type: promoted_type
                });
            },

            Punctuator::PERCENT => {
                result.add_comment("calculating modulus");

                result.merge(&put_lhs_ax_rhs_cx(&self.lhs, &self.rhs, &promoted_type, asm_data, stack_data));

                unwrap_let!(DataType::RAW(_) = promoted_type);

                result.add_instruction(AsmOperation::DIV {
                    divisor: Operand::Register(AsmRegister::secondary()),
                    data_type: promoted_type
                });

                //mod is returned in RDX
                result.add_instruction(AsmOperation::MOV { to: Operand::Register(AsmRegister::acc()), from: Operand::Register(AsmRegister::_DX), size: promoted_size });
            }

            comparison if comparison.as_comparator_instr().is_some() => { // >, <, ==, >=, <=
                result.add_comment("comparing numbers");
                result.merge(&put_lhs_ax_rhs_cx(&self.lhs, &self.rhs, &promoted_type, asm_data, stack_data));

                result.add_instruction(AsmOperation::CMP {
                    lhs: Operand::Register(AsmRegister::acc()),
                    rhs: Operand::Register(AsmRegister::secondary()),
                    data_type: promoted_type
                });

                let asm_comparison = comparison.as_comparator_instr().unwrap();

                //create the correct setcc instruction
                result.add_instruction(AsmOperation::SETCC { destination: Operand::Register(AsmRegister::acc()), comparison: asm_comparison });

            },

            operator if operator.as_boolean_instr().is_some() => {

                //perhaps this will work for binary operators too?
                //warning: what if either side is not a boolean
                result.add_comment("applying boolean operator");

                result.merge(&put_lhs_ax_rhs_cx(&self.lhs, &self.rhs, &promoted_type, asm_data, stack_data));//also casts too boolean

                unwrap_let!(DataType::RAW(promoted_underlying) = promoted_type);
                assert!(promoted_underlying.is_integer());//floating point division??

                assert!(promoted_underlying.memory_size(asm_data).size_bytes() == 1);//must be boolean
                assert!(promoted_underlying == BaseType::_BOOL);

                let boolean_instruction = operator.as_boolean_instr().unwrap();

                result.add_instruction(AsmOperation::BooleanOp {
                    destination: Operand::Register(AsmRegister::acc()),
                    secondary: Operand::Register(AsmRegister::secondary()),
                    operation: boolean_instruction
                });
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
            Punctuator::EXCLAMATIONEQUALS => DataType::RAW(BaseType::_BOOL),

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