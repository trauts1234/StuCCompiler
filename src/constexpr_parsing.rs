use crate::{assembly::comparison::ComparisonKind, binary_expression::BinaryExpression, data_type::base_type::IntegerType, debugging::IRDisplay, expression::{binary_expression_operator::BinaryExpressionOperator, expression::Expression, ternary::TernaryExpr, unary_prefix_expr::UnaryPrefixExpression, unary_prefix_operator::UnaryPrefixOperator}, number_literal::typed_value::NumberLiteral, string_literal::StringLiteral};

#[derive(Debug)]
pub enum ConstexprValue {
    NUMBER(NumberLiteral),
    STRING(StringLiteral),
    POINTER{label: String, offset: NumberLiteral},
    ZEROES,//sets the memory to all 0 (zero initialisation)
}

impl TryFrom<&Expression> for ConstexprValue {
    
    type Error = String;
    
    fn try_from(value: &Expression) -> Result<Self, Self::Error> {
        match value {
            Expression::NUMBERLITERAL(number_literal) => Ok(ConstexprValue::NUMBER(number_literal.clone())),
            Expression::VARIABLE(minimal_data_variable) => Err(format!("variable {} is not a compile-time constant", minimal_data_variable.name)),
            Expression::STRUCTMEMBERACCESS(struct_member_access) => todo!(),
            Expression::STRINGLITERAL(string_literal) => Ok(ConstexprValue::STRING(string_literal.clone())),
            Expression::ARRAYLITERAL(array_initialisation) => todo!(),
            Expression::FUNCCALL(function_call) => Err(format!("results of calling {} are not a compile time constant", function_call.get_callee_decl().function_name)),
            Expression::UNARYPREFIX(unary_prefix_expression) => unary_prefix_expression.try_into(),
            Expression::UNARYSUFFIX(_) => Err("cannot fold unary postfix increment/decrement".to_owned()),
            Expression::BINARYEXPRESSION(binary_expression) => binary_expression.clone().try_into(),
            Expression::CAST(cast_expression) => todo!(),
            Expression::SIZEOF(sizeof_expr) => Err(format!("no asm_data in constant folding, so cannot evaluate sizeof")),//sizeof
            Expression::TERNARYEXPRESSION(ternary) => ternary.clone().try_into(),
        }
    }
}

impl TryFrom<&UnaryPrefixExpression> for ConstexprValue {
    type Error = String;

    fn try_from(value: &UnaryPrefixExpression) -> Result<Self, Self::Error> {

        if let (UnaryPrefixOperator::Reference, Expression::VARIABLE(var)) = (value.get_operator(), value.get_operand()) {
            //getting address of variable
            return Ok(ConstexprValue::POINTER { label: var.name.to_string(), offset: NumberLiteral::from(0) })
        }

        let operand: ConstexprValue = value.get_operand().try_into()?;
        match (value.get_operator(), operand) {
            (UnaryPrefixOperator::Reference, _) => Err("cannot get address of this".to_owned()),
            (UnaryPrefixOperator::Dereference, _) => Err("cannot dereference pointer in constant expression".to_owned()),
            (UnaryPrefixOperator::Negate, ConstexprValue::NUMBER(num)) => Ok(ConstexprValue::NUMBER(-num)),
            (UnaryPrefixOperator::UnaryPlus, ConstexprValue::NUMBER(num)) => Ok(ConstexprValue::NUMBER(num.unary_plus())),
            (UnaryPrefixOperator::BitwiseNot, ConstexprValue::NUMBER(num)) => Ok(ConstexprValue::NUMBER(num.bitwise_not())),
            (UnaryPrefixOperator::BooleanNot, ConstexprValue::NUMBER(num)) => Ok(ConstexprValue::NUMBER(num.boolean_not())),

            _ =>todo!("this unary punctuator is not implemented for constant folding")
        }
    }
}

impl TryFrom<BinaryExpression> for ConstexprValue {
    type Error = String;

    fn try_from(value: BinaryExpression) -> Result<Self, Self::Error> {
        let lhs: ConstexprValue = value.lhs().try_into()?;
        let rhs: ConstexprValue = value.rhs().try_into()?;
        match (lhs, value.operator(), rhs) {
            (ConstexprValue::NUMBER(l), BinaryExpressionOperator::BitwiseOr, ConstexprValue::NUMBER(r)) => Ok(ConstexprValue::NUMBER(l | r)),
            (ConstexprValue::NUMBER(l), BinaryExpressionOperator::BitwiseAnd, ConstexprValue::NUMBER(r)) => Ok(ConstexprValue::NUMBER(l & r)),
            (ConstexprValue::NUMBER(l), BinaryExpressionOperator::BitwiseXor, ConstexprValue::NUMBER(r)) => Ok(ConstexprValue::NUMBER(l ^ r)),
            (ConstexprValue::NUMBER(l), BinaryExpressionOperator::Add, ConstexprValue::NUMBER(r)) => Ok(ConstexprValue::NUMBER(l + r)),
            (ConstexprValue::NUMBER(l), BinaryExpressionOperator::Subtract, ConstexprValue::NUMBER(r)) => Ok(ConstexprValue::NUMBER(l - r)),
            (ConstexprValue::NUMBER(l), BinaryExpressionOperator::Multiply, ConstexprValue::NUMBER(r)) => Ok(ConstexprValue::NUMBER(l * r)),
            (ConstexprValue::NUMBER(l), BinaryExpressionOperator::Divide, ConstexprValue::NUMBER(r)) => Ok(ConstexprValue::NUMBER(l / r)),
            (ConstexprValue::NUMBER(l), BinaryExpressionOperator::Mod, ConstexprValue::NUMBER(r)) => Ok(ConstexprValue::NUMBER(l % r)),
            (ConstexprValue::NUMBER(l), BinaryExpressionOperator::BitshiftLeft, ConstexprValue::NUMBER(r)) => Ok(ConstexprValue::NUMBER(l << r)),
            (ConstexprValue::NUMBER(l), BinaryExpressionOperator::BitshiftRight, ConstexprValue::NUMBER(r)) => Ok(ConstexprValue::NUMBER(l >> r)),

            (ConstexprValue::NUMBER(l), BinaryExpressionOperator::BooleanOr, ConstexprValue::NUMBER(r)) => Ok(ConstexprValue::NUMBER(l.boolean_or(r))),
            (ConstexprValue::NUMBER(l), BinaryExpressionOperator::BooleanAnd, ConstexprValue::NUMBER(r)) => Ok(ConstexprValue::NUMBER(l.boolean_and(r))),

            (ConstexprValue::NUMBER(l), op, ConstexprValue::NUMBER(r)) if op.as_comparator_instr().is_some() =>
                Ok(ConstexprValue::NUMBER(NumberLiteral::INTEGER{
                    data: l.cmp(r, &op.as_comparator_instr().unwrap()) as i128,
                    data_type: IntegerType::_BOOL
                })),
            
            _ => todo!("constexpr folding of binary operator {:?}", value.operator())
        }
    }
}

impl TryFrom<TernaryExpr> for ConstexprValue {
    type Error = String;

    fn try_from(value: TernaryExpr) -> Result<Self, Self::Error> {
        let true_branch: ConstexprValue = value.true_branch().try_into()?;
        let false_branch: ConstexprValue = value.false_branch().try_into()?;
        let condition: ConstexprValue = value.condition().try_into()?;

        let condition_true = match condition {
            ConstexprValue::NUMBER(num) => num.cmp(NumberLiteral::INTEGER { data: 0, data_type: IntegerType::I32 }, &ComparisonKind::NE),
            _ => todo!("compare this data type")
        };

        Ok(if condition_true {
            true_branch
        } else {
            false_branch
        })
    }
}

impl IRDisplay for ConstexprValue {
    fn display_ir(&self) -> String {
        match self {
            ConstexprValue::NUMBER(number_literal) => format!("{}", number_literal),
            ConstexprValue::STRING(string_literal) => format!("{}", string_literal),
            ConstexprValue::POINTER { label, offset } => format!("&({} + {})", label, offset),
            ConstexprValue::ZEROES => "0".to_owned(),
        }
    }
}