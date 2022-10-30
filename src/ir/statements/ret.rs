use std::fmt;

use crate::ir::quantity::{local_or_number_literal, LocalOrNumberLiteral};
use nom::{
    bytes::complete::tag,
    character::complete::space0,
    combinator::{map, opt},
    sequence::tuple,
    IResult,
};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Ret {
    pub value: Option<LocalOrNumberLiteral>,
}

impl fmt::Display for Ret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(value) = &self.value {
            write!(f, "ret {}", value)
        } else {
            write!(f, "ret")
        }
    }
}

pub fn parse(code: &str) -> IResult<&str, Ret> {
    map(
        tuple((tag("ret"), space0, opt(local_or_number_literal))),
        |(_, _, value)| Ret { value },
    )(code)
}
