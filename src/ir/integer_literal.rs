use crate::{ast, utility::parsing};
use nom::{combinator::map, IResult};

/// An integer literal.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct IntegerLiteral(pub i64);

impl From<i64> for IntegerLiteral {
    fn from(i: i64) -> Self {
        IntegerLiteral(i)
    }
}

impl From<ast::expression::integer_literal::IntegerLiteral> for IntegerLiteral {
    fn from(i: ast::expression::IntegerLiteral) -> Self {
        Self(i.0)
    }
}

/// Parse ir code to get an [`IntegerLiteral`].
pub fn parse(code: &str) -> IResult<&str, IntegerLiteral> {
    map(parsing::integer, IntegerLiteral)(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!(parse("123"), Ok(("", IntegerLiteral(123))));
        assert_eq!(parse("0"), Ok(("", IntegerLiteral(0))));
        assert_eq!(parse("-123"), Ok(("", IntegerLiteral(-123))));
        assert_eq!(parse("123abc"), Ok(("abc", IntegerLiteral(123))));
        assert!(parse("abc").is_err());
    }
}
