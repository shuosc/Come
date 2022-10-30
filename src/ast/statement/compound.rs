use super::Statement;
use crate::{ast::statement, utility::parsing};
use nom::{bytes::complete::tag, combinator::map, multi::many0, sequence::delimited, IResult};

/// [`Compound`] represents an group of statements wrapped in `{` and `}`
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Compound(pub Vec<Statement>);

/// Parse source code to get an [`Compound`].
pub fn parse(code: &str) -> IResult<&str, Compound> {
    map(
        delimited(
            tag("{"),
            many0(parsing::in_multispace(statement::parse)),
            tag("}"),
        ),
        Compound,
    )(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse() {
        let assign = parse(
            r#"{
            a = 1;
            b = 2;
        }"#,
        )
        .unwrap()
        .1;
        assert_eq!(assign.0.len(), 2);
        let assign = parse("{}").unwrap().1;
        assert_eq!(assign.0.len(), 0);
    }
}
