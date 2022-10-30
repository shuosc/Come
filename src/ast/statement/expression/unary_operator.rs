use super::{
    field_access::{self, higher_than_field_access},
    rvalue::RValue,
};
use nom::{branch::alt, bytes::complete::tag, combinator::map, sequence::tuple, IResult};

/// [`UnaryOperatorResult`] represents result of a unary operator.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct UnaryOperatorResult {
    /// The operator.
    pub operator: String,
    /// The operand.
    pub operand: Box<RValue>,
}

pub fn higher_than_unary_operator_result(code: &str) -> IResult<&str, RValue> {
    alt((
        map(field_access::parse, RValue::FieldAccess),
        higher_than_field_access,
    ))(code)
}

/// Parse source code to get a [`UnaryOperatorResult`].
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse() {
        let result = parse("-a").unwrap().1;
        assert_eq!(result.operator, "-");
    }
}
