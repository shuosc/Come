use nom::{
    bytes::complete::tag, character::complete::space0, combinator::map, sequence::tuple, IResult,
};

use crate::ast::expression::rvalue::{self, RValue};

use super::compound::{self, Compound};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct While {
    pub condition: RValue,
    pub content: Compound,
}

pub fn parse(code: &str) -> IResult<&str, While> {
    map(
        tuple((tag("while"), space0, rvalue::parse, space0, compound::parse)),
        |(_, _, condition, _, content)| While { condition, content },
    )(code)
}
