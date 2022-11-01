use crate::utility::parsing;
use nom::{combinator::map, IResult};

/// [`IntegerLiteral`] represents an integer literal.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct IntegerLiteral(pub i64);

impl From<i64> for IntegerLiteral {
    fn from(i: i64) -> Self {
        IntegerLiteral(i)
    }
}

/// Parse source code to get a [`IntegerLiteral`].
pub fn parse(code: &str) -> IResult<&str, IntegerLiteral> {
    map(parsing::integer, IntegerLiteral)(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse() {
        let function_call = parse("123").unwrap().1;
        assert_eq!(function_call.0, 123);
        let function_call = parse("-0xabcd").unwrap().1;
        assert_eq!(function_call.0, -0xabcd);
    }
}
