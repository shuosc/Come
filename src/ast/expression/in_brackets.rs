use nom::{bytes::complete::tag, combinator::map, sequence::delimited, IResult};

use super::rvalue::{self, RValue};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct InBrackets(pub Box<RValue>);

pub fn parse(code: &str) -> IResult<&str, InBrackets> {
    map(delimited(tag("("), rvalue::parse, tag(")")), |content| {
        InBrackets(Box::new(content))
    })(code)
}

// todo: test
