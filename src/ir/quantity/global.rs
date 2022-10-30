use crate::utility::parsing;
use nom::{bytes::complete::tag, combinator::map, sequence::pair, IResult};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Global(pub String);

impl Display for Global {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "@{}", self.0)
    }
}

pub fn parse(code: &str) -> IResult<&str, Global> {
    map(pair(tag("@"), parsing::ident), |(_, name)| Global(name))(code)
}
