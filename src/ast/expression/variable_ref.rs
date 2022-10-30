use nom::{combinator::map, IResult};

use crate::utility::parsing;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct VariableRef(pub String);

pub fn parse(code: &str) -> IResult<&str, VariableRef> {
    map(parsing::ident, VariableRef)(code)
}

// todo: test
