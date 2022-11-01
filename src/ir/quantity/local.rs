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

/// [`LocalVariableName`] represents a local variable's name.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct LocalVariableName(pub String);

impl Display for LocalVariableName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}

/// Parse source code to get a [`LocalVariableName`].
pub fn parse(code: &str) -> IResult<&str, LocalVariableName> {
    map(
        pair(tag("%"), alt((digit1, recognize(parsing::ident)))),
        |(_, name)| LocalVariableName(name.to_string()),
    )(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let result = parse("%foo").unwrap().1;
        assert_eq!(result.0, "foo");
        let result = parse("%0").unwrap().1;
        assert_eq!(result.0, "0");
    }
}
