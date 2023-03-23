use super::{
    binary_operator::{self, BinaryOperatorResult},
    field_access::FieldAccess,
    function_call::FunctionCall,
    in_brackets::{InBrackets, self},
    integer_literal::IntegerLiteral,
    lvalue::LValue,
    unary_operator::{self, UnaryOperatorResult},
    variable_ref::VariableRef,
};
use enum_dispatch::enum_dispatch;
use nom::{branch::alt, combinator::map, IResult};

/// Tag trait for [`RValue`].
#[enum_dispatch]
trait IsRValue {}

/// [`RValue`] represents an expression that has a value.
#[enum_dispatch(IsRValue)]
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum RValue {
    IntegerLiteral,
    VariableRef,
    InBrackets,
    FieldAccess,
    FunctionCall,
    UnaryOperatorResult,
    BinaryOperatorResult,
}

/// Parse source code to get a [`RValue`].
pub fn parse(code: &str) -> IResult<&str, RValue> {
    alt((
        map(in_brackets::parse, RValue::InBrackets),
        map(binary_operator::parse, RValue::BinaryOperatorResult),
        map(unary_operator::parse, RValue::UnaryOperatorResult),
        unary_operator::higher_than_unary_operator_result,
    ))(code)
}

impl From<LValue> for RValue {
    fn from(lvalue: LValue) -> Self {
        match lvalue {
            LValue::VariableRef(variable_ref) => RValue::VariableRef(variable_ref),
            LValue::FieldAccess(field_access) => RValue::FieldAccess(field_access),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse() {
        let rvale = super::parse("42").unwrap().1;
        assert!(matches!(rvale, RValue::IntegerLiteral(_)));

        let rvale = super::parse("a").unwrap().1;
        assert!(matches!(rvale, RValue::VariableRef(_)));

        let rvalue = super::parse("(a+b)").unwrap().1;
        assert!(matches!(rvalue, RValue::InBrackets(_)));

        let rvalue = super::parse("a.b").unwrap().1;
        assert!(matches!(rvalue, RValue::FieldAccess(_)));

        let rvalue = super::parse("f(a, b, c)").unwrap().1;
        assert!(matches!(rvalue, RValue::FunctionCall(_)));

        let rvalue = super::parse("-a").unwrap().1;
        assert!(matches!(rvalue, RValue::UnaryOperatorResult(_)));

        let rvalue = super::parse("a + b").unwrap().1;
        assert!(matches!(rvalue, RValue::BinaryOperatorResult(_)));
    }
}
