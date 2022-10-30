use nom::{bytes::complete::tag, combinator::map, multi::many0, sequence::delimited, IResult};

use crate::utility::parsing;

use crate::ast::statement;

use super::Statement;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Compound(pub Vec<Statement>);

pub fn parse(code: &str) -> IResult<&str, Compound> {
    map(
        delimited(
            tag("{"),
            many0(parsing::in_multispace(statement::parse)),
            tag("}"),
        ),
        Compound,
    )(code)
}
