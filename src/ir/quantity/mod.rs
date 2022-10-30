pub mod global;
pub mod local;

pub use crate::ir::quantity::{global::Global, local::Local};
use crate::utility::parsing;
use enum_dispatch::enum_dispatch;
use nom::{branch::alt, combinator::map, IResult};
use std::fmt::{self, Display, Formatter};

#[enum_dispatch]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum LocalOrNumberLiteral {
    Local,
    NumberLiteral(i64),
}

pub fn local_or_number_literal(code: &str) -> IResult<&str, LocalOrNumberLiteral> {
    alt((
        map(local::parse, LocalOrNumberLiteral::Local),
        map(parsing::integer, LocalOrNumberLiteral::NumberLiteral),
    ))(code)
}

impl Display for LocalOrNumberLiteral {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LocalOrNumberLiteral::Local(local) => write!(f, "{}", local),
            LocalOrNumberLiteral::NumberLiteral(number) => write!(f, "{}", number),
        }
    }
}

#[enum_dispatch]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum LocalOrGlobal {
    Local,
    Global,
}

pub fn local_or_global(code: &str) -> IResult<&str, LocalOrGlobal> {
    alt((
        map(local::parse, LocalOrGlobal::Local),
        map(global::parse, LocalOrGlobal::Global),
    ))(code)
}

impl Display for LocalOrGlobal {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LocalOrGlobal::Local(local) => write!(f, "{}", local),
            LocalOrGlobal::Global(global) => write!(f, "{}", global),
        }
    }
}
