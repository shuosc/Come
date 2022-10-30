use nom::{
    bytes::complete::tag,
    character::complete::{multispace0, multispace1},
    combinator::{map, opt},
    sequence::{pair, tuple},
    IResult,
};

use crate::ast::expression::rvalue::{self, RValue};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Return(pub Option<RValue>);

pub fn parse(code: &str) -> IResult<&str, Return> {
    map(
        tuple((
            tag("return"),
            opt(pair(multispace1, rvalue::parse)),
            multispace0,
            tag(";"),
        )),
        |(_, value, _, _)| Return(value.map(|it| it.1)),
    )(code)
}
