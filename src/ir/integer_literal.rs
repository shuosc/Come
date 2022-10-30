use crate::{ast, utility::parsing};
use nom::{combinator::map, IResult};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct IntegerLiteral(pub i64);

impl From<i64> for IntegerLiteral {
    fn from(i: i64) -> Self {
        IntegerLiteral(i)
    }
}

impl From<ast::expression::integer_literal::IntegerLiteral> for IntegerLiteral {
    fn from(i: ast::expression::integer_literal::IntegerLiteral) -> Self {
        Self(i.0)
    }
}

pub fn parse(code: &str) -> IResult<&str, IntegerLiteral> {
    map(parsing::integer, IntegerLiteral)(code)
}
