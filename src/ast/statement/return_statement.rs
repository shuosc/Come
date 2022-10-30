use super::expression::rvalue::{self, RValue};
use nom::{
    bytes::complete::tag,
    character::complete::{multispace0, multispace1},
    combinator::{map, opt},
    sequence::{pair, tuple},
    IResult,
};

/// [`Return`] represents an `return` statement, with an optional return value.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Return(pub Option<RValue>);

/// Parse source code to get a [`Return`].
pub fn parse(code: &str) -> IResult<&str, Return> {
    map(
        tuple((
            tag("return"),
            opt(pair(multispace1, rvalue::parse)),
            multispace0,
            tag(";"),
        )),
        |(_, value, _, _)| Return(value.map(|it| it.1)),
    )(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expression::VariableRef;

    #[test]
    fn can_parse() {
        let return_statement = parse("return;").unwrap().1;
        assert_eq!(return_statement.0, None);

        let return_statement = parse("return a;").unwrap().1;
        assert_eq!(
            return_statement.0,
            Some(VariableRef("a".to_string()).into())
        );
    }
}
