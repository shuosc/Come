use std::fmt;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{map, recognize},
    sequence::pair,
    IResult,
};

use super::parsing;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Integer {
    pub signed: bool,
    pub width: usize,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Type {
    Integer(Integer),
    StructRef(String),
    None,
    Address,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Integer(i) => i.fmt(f),
            Type::Address => write!(f, "address"),
            Type::StructRef(name) => write!(f, "{}", name),
            Type::None => write!(f, "()"),
        }
    }
}

impl From<Integer> for Type {
    fn from(integer: Integer) -> Self {
        Type::Integer(integer)
    }
}

pub fn parse_integer(code: &str) -> IResult<&str, Integer> {
    alt((
        map(pair(tag("i"), digit1), |(_, width_str): (_, &str)| {
            Integer {
                signed: true,
                width: width_str.parse::<usize>().unwrap(),
            }
        }),
        map(pair(tag("u"), digit1), |(_, width_str): (_, &str)| {
            Integer {
                signed: false,
                width: width_str.parse::<usize>().unwrap(),
            }
        }),
    ))(code)
}

pub fn parse(code: &str) -> IResult<&str, Type> {
    alt((
        map(
            alt((recognize(pair(parse_integer, tag("*"))), tag("address"))),
            |_| Type::Address,
        ),
        map(parse_integer, Type::Integer),
        map(parsing::ident, Type::StructRef),
        map(tag("()"), |_| Type::None),
    ))(code)
}

impl fmt::Display for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", if self.signed { "i" } else { "u" }, self.width)
    }
}
