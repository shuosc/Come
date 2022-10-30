use enum_dispatch::enum_dispatch;
use nom::{branch::alt, combinator::map, IResult};

use super::{
    binary_operator::{self, BinaryOperatorResult},
    field_access::FieldAccess,
    function_call::FunctionCall,
    in_brackets::InBrackets,
    integer_literal::IntegerLiteral,
    unary_operator::{self, UnaryOperatorResult},
    variable_ref::VariableRef,
};

#[enum_dispatch]
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

pub fn parse(code: &str) -> IResult<&str, RValue> {
    alt((
        map(binary_operator::parse, RValue::BinaryOperatorResult),
        map(unary_operator::parse, RValue::UnaryOperatorResult),
        unary_operator::higher_than_unary_operator_result,
    ))(code)
}

// todo: test
