use crate::utility::parsing;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{map, recognize},
    sequence::pair,
    IResult,
};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Local(pub String);

impl Display for Local {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}

pub fn parse(code: &str) -> IResult<&str, Local> {
    map(
        pair(tag("%"), alt((digit1, recognize(parsing::ident)))),
        |(_, name)| Local(name.to_string()),
    )(code)
}
