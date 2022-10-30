use nom::{branch::alt, bytes::complete::tag, combinator::map, sequence::tuple, IResult};

use super::{
    field_access::{self, higher_than_field_access},
    rvalue::RValue,
};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct UnaryOperatorResult {
    pub operator: String,
    pub operand: Box<RValue>,
}

pub(in crate::ast::expression) fn higher_than_unary_operator_result(
    code: &str,
) -> IResult<&str, RValue> {
    alt((
        map(field_access::parse, RValue::FieldAccess),
        higher_than_field_access,
    ))(code)
}

pub fn parse(code: &str) -> IResult<&str, UnaryOperatorResult> {
    map(
        tuple((
            alt((tag("+"), tag("-"), tag("!"), tag("~"))),
            higher_than_unary_operator_result,
        )),
        |(op, operand)| UnaryOperatorResult {
            operator: op.to_string(),
            operand: Box::new(operand),
        },
    )(code)
}

// todo: test
