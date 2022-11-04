use super::{
    binary_operator::{self, BinaryOperatorResult},
    field_access::FieldAccess,
    function_call::FunctionCall,
    in_brackets::InBrackets,
    integer_literal::IntegerLiteral,
    lvalue::LValue,
    unary_operator::{self, UnaryOperatorResult},
    variable_ref::VariableRef,
};
use enum_dispatch::enum_dispatch;
use nom::{branch::alt, combinator::map, IResult};

/// Tag trait for [`RValue`].
#[enum_dispatch]
pub trait IsRValue {}

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

// todo: test
