use nom::{
    bytes::complete::tag, character::complete::multispace0, combinator::map, sequence::tuple,
    IResult,
};

use crate::{
    ast::expression::{
        lvalue::{self, LValue},
        rvalue::{self, RValue},
    },
    utility::parsing::in_multispace,
};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Assign {
    pub lhs: LValue,
    pub rhs: RValue,
}

pub fn parse(code: &str) -> IResult<&str, Assign> {
    map(
        tuple((
            lvalue::parse,
            in_multispace(tag("=")),
            rvalue::parse,
            multispace0,
            tag(";"),
        )),
        |(lhs, _, rhs, _, _)| Assign { lhs, rhs },
    )(code)
}
