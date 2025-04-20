use crate::{binary_expression::BinaryExpression, debugging::{DebugDisplay, IRDisplay}, expression::Expression, lexer::punctuator::Punctuator, number_literal::typed_value::NumberLiteral, string_literal::StringLiteral, unary_prefix_expr::UnaryPrefixExpression};

pub enum ConstexprValue {
    NUMBER(NumberLiteral),
    STRING(StringLiteral),
    POINTER{label: String, offset: NumberLiteral},
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
            Expression::UNARYPREFIX(unary_prefix_expression) => unary_prefix_expression.clone().try_into(),
            Expression::BINARYEXPRESSION(binary_expression) => binary_expression.clone().try_into(),
            Expression::CAST(cast_expression) => todo!(),
        }
    }
}

impl TryFrom<UnaryPrefixExpression> for ConstexprValue {
    type Error = String;

    fn try_from(value: UnaryPrefixExpression) -> Result<Self, Self::Error> {

        if let (Punctuator::AMPERSAND, Expression::VARIABLE(var)) = (value.get_operator(), value.get_operand()) {
            //getting address of variable
            return Ok(ConstexprValue::POINTER { label: var.name.to_string(), offset: NumberLiteral::from(0) })
        }

        let operand: ConstexprValue = value.get_operand().try_into()?;
        match (value.get_operator(), operand) {
            (Punctuator::AMPERSAND, _) => Err("cannot get address of this".to_owned()),
            (Punctuator::ASTERISK, _) => Err("cannot dereference pointer in constant expression".to_owned()),
            (Punctuator::DASH, ConstexprValue::NUMBER(num)) => Ok(ConstexprValue::NUMBER(-num)),
            (Punctuator::PLUS, ConstexprValue::NUMBER(num)) => Ok(ConstexprValue::NUMBER(num.unary_plus())),
            (Punctuator::Tilde, ConstexprValue::NUMBER(num)) => Ok(ConstexprValue::NUMBER(num.bitwise_not())),
            (Punctuator::Exclamation, ConstexprValue::NUMBER(num)) => Ok(ConstexprValue::NUMBER(num.boolean_not())),

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
            (ConstexprValue::NUMBER(l), Punctuator::Pipe, ConstexprValue::NUMBER(r)) => Ok(ConstexprValue::NUMBER(l | r)),
            (ConstexprValue::NUMBER(l), Punctuator::AMPERSAND, ConstexprValue::NUMBER(r)) => Ok(ConstexprValue::NUMBER(l & r)),
            (ConstexprValue::NUMBER(l), Punctuator::Hat, ConstexprValue::NUMBER(r)) => Ok(ConstexprValue::NUMBER(l ^ r)),
            (ConstexprValue::NUMBER(l), Punctuator::PLUS, ConstexprValue::NUMBER(r)) => Ok(ConstexprValue::NUMBER(l + r)),
            (ConstexprValue::NUMBER(l), Punctuator::DASH, ConstexprValue::NUMBER(r)) => Ok(ConstexprValue::NUMBER(l - r)),
            (ConstexprValue::NUMBER(l), Punctuator::ASTERISK, ConstexprValue::NUMBER(r)) => Ok(ConstexprValue::NUMBER(l * r)),
            (ConstexprValue::NUMBER(l), Punctuator::FORWARDSLASH, ConstexprValue::NUMBER(r)) => Ok(ConstexprValue::NUMBER(l / r)),
            (ConstexprValue::NUMBER(l), Punctuator::PERCENT, ConstexprValue::NUMBER(r)) => Ok(ConstexprValue::NUMBER(l % r)),
            (ConstexprValue::NUMBER(l), Punctuator::LessLess, ConstexprValue::NUMBER(r)) => Ok(ConstexprValue::NUMBER(l << r)),
            (ConstexprValue::NUMBER(l), Punctuator::GreaterGreater, ConstexprValue::NUMBER(r)) => Ok(ConstexprValue::NUMBER(l >> r)),

            (ConstexprValue::NUMBER(l), op, ConstexprValue::NUMBER(r)) if op.as_comparator_instr().is_some() => Ok(ConstexprValue::NUMBER(l.cmp(r, &op.as_comparator_instr().unwrap()))),
            

            _ => todo!("constexpr folding of binary operator: {:?}", value.operator())
        }
    }
}

impl IRDisplay for ConstexprValue {
    fn display_ir(&self) -> String {
        match self {
            ConstexprValue::NUMBER(number_literal) => number_literal.display(),
            ConstexprValue::STRING(string_literal) => string_literal.display(),
            ConstexprValue::POINTER { label, offset } => format!("&({} + {})", label, offset.display()),
        }
    }
}