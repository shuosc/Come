use super::rvalue::{self, RValue};
use nom::{bytes::complete::tag, combinator::map, sequence::delimited, IResult};

/// [`InBrackets`] represents an expression in brackets.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct InBrackets(pub Box<RValue>);

/// Parse source code to get a [`InBrackets`].
pub fn parse(code: &str) -> IResult<&str, InBrackets> {
    map(delimited(tag("("), rvalue::parse, tag(")")), |content| {
        InBrackets(Box::new(content))
    })(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse() {
        assert!(parse("(a+b)").is_ok());
        assert!(parse("(a+b").is_err());
    }
}
