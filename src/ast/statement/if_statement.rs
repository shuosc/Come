use nom::{
    bytes::complete::tag,
    character::complete::{space0, space1},
    combinator::{map, opt},
    sequence::tuple,
    IResult,
};

use crate::{
    ast::expression::rvalue::{self, RValue},
    utility::parsing,
};

use super::compound::{self, Compound};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct If {
    pub condition: RValue,
    pub content: Compound,
    pub else_content: Option<Compound>,
}

pub fn parse(code: &str) -> IResult<&str, If> {
    map(
        tuple((
            tag("if"),
            space1,
            rvalue::parse,
            space0,
            compound::parse,
            opt(map(
                tuple((parsing::in_multispace(tag("else")), compound::parse)),
                |(_, else_content)| else_content,
            )),
        )),
        |(_, _, condition, _, content, else_content)| If {
            condition,
            content,
            else_content,
        },
    )(code)
}
