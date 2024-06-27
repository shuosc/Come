use crate::utility::parsing;
use nom::{bytes::complete::tag, combinator::map, sequence::pair, IResult};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// [`GlobalVariableName`] represents a global variable's name.
#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct GlobalVariableName(pub String);

impl Display for GlobalVariableName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "@{}", self.0)
    }
}

/// Parse source code to get a [`GlobalVariableName`].
pub fn parse(code: &str) -> IResult<&str, GlobalVariableName> {
    map(pair(tag("@"), parsing::ident), |(_, name)| {
        GlobalVariableName(name)
    })(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let result = parse("@foo").unwrap().1;
        assert_eq!(result.0, "foo");
    }
}
