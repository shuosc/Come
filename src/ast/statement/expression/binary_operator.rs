use super::rvalue::RValue;
use nom::{branch::alt, IResult};
use paste::paste;

/// [`BinaryOperatorResult`] represents result of a binary operator.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct BinaryOperatorResult {
    /// The operator.
    pub operator: String,
    /// The left hand side of the operator.
    pub lhs: Box<RValue>,
    /// The right hand side of the operator.
    pub rhs: Box<RValue>,
}

mod level3 {
    use super::*;
    use crate::{
        ast::expression::{
            rvalue::RValue,
            unary_operator::{self, higher_than_unary_operator_result},
        },
        utility::parsing,
    };
    use nom::{
        branch::alt, bytes::complete::tag, combinator::map, multi::fold_many0, sequence::pair,
        IResult,
    };

    pub fn higher_than_level3(code: &str) -> IResult<&str, RValue> {
        alt((
            map(unary_operator::parse, RValue::UnaryOperatorResult),
            higher_than_unary_operator_result,
        ))(code)
    }

    pub fn parse(code: &str) -> IResult<&str, BinaryOperatorResult> {
        let (rest, lhs) = higher_than_level3(code)?;
        let (rest, operator) = parsing::in_multispace(alt((tag("*"), tag("/"))))(rest)?;
        let (rest, rhs) = higher_than_level3(rest)?;
        fold_many0(
            pair(
                parsing::in_multispace(alt((tag("*"), tag("/")))),
                higher_than_level3,
            ),
            move || BinaryOperatorResult {
                operator: operator.to_string(),
                lhs: Box::new(lhs.clone()),
                rhs: Box::new(rhs.clone()),
            },
            |lhs, (operator, rhs)| BinaryOperatorResult {
                operator: operator.to_string(),
                lhs: Box::new(RValue::BinaryOperatorResult(lhs)),
                rhs: Box::new(rhs),
            },
        )(rest)
    }
}

/// A macro for generating binary operator parsers.
/// The `level` here are from C's operator precedence.
macro_rules! bin_op_level {
    ($n: expr, $n_minus_1: expr, $($op: expr)*) => {
        paste! {
        mod [<level $n>] {
            use super::*;
            use nom::{
                branch::alt, bytes::complete::tag, combinator::map, multi::fold_many0,
                sequence::pair, IResult,
            };
            use crate::{
                ast::expression::{rvalue::RValue, binary_operator::[<level $n_minus_1>]::{self, [<higher_than_level $n_minus_1>]}},
                utility::parsing,
            };

            pub(in crate::ast::expression) fn [<higher_than_level $n>](
                code: &str,
            ) -> IResult<&str, RValue> {
                alt((
                    map([<level $n_minus_1>]::parse, RValue::BinaryOperatorResult),
                    [<higher_than_level $n_minus_1>],
                ))(code)
            }

            pub fn parse(code: &str) -> IResult<&str, BinaryOperatorResult> {
                let (rest, lhs) = [<higher_than_level $n>](code)?;
                let (rest, operator) = parsing::in_multispace(alt(($(tag($op),)*)))(rest)?;
                let (rest, rhs) = [<higher_than_level $n>](rest)?;
                fold_many0(
                    pair(
                        parsing::in_multispace(alt(($(tag($op),)*))),
                        [<higher_than_level $n>],
                    ),
                    move || BinaryOperatorResult {
                        operator: operator.to_string(),
                        lhs: Box::new(lhs.clone()),
                        rhs: Box::new(rhs.clone()),
                    },
                    |lhs, (operator, rhs)| BinaryOperatorResult {
                        operator: operator.to_string(),
                        lhs: Box::new(RValue::BinaryOperatorResult(lhs)),
                        rhs: Box::new(rhs),
                    },
                )(rest)
            }
        }
    }
    };
}

bin_op_level!(4, 3, "+" "-");
bin_op_level!(5, 4, "<<" ">>");
bin_op_level!(6, 5, "<=" "<" ">=" ">");
bin_op_level!(7, 6, "==" "!=");
bin_op_level!(8, 7, "&" "&");
bin_op_level!(9, 8, "^" "^");
bin_op_level!(10, 9, "|" "|");

/// Parse source code to get a [`BinaryOperatorResult`].
pub fn parse(code: &str) -> IResult<&str, BinaryOperatorResult> {
    alt((
        level10::parse,
        level9::parse,
        level8::parse,
        level7::parse,
        level6::parse,
        level5::parse,
        level4::parse,
        level3::parse,
    ))(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn can_parse() {
        let bin_op = parse("s.a + s.b").unwrap().1;
        assert_eq!(bin_op.operator, "+");
        let bin_op = parse("a+b*c").unwrap().1;
        assert_eq!(bin_op.operator, "+");
        let bin_op = parse("b*c+d").unwrap().1;
        assert_eq!(bin_op.operator, "+");
        let bin_op = parse("!b+d").unwrap().1;
        assert_eq!(bin_op.operator, "+");
    }
}
