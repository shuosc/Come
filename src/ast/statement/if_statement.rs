use super::{
    compound::{self, Compound},
    expression::rvalue::{self, RValue},
};
use crate::utility::parsing;
use nom::{
    bytes::complete::tag,
    character::complete::{space0, space1},
    combinator::{map, opt},
    sequence::tuple,
    IResult,
};

/// [`If`] represents an `if` statement.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct If {
    /// The condition of the `if` statement.
    pub condition: RValue,
    /// The body of the `if` statement.
    pub content: Compound,
    /// The body of the corresponding `else` statement.
    pub else_content: Option<Compound>,
}

/// Parse source code to get an [`If`].
pub fn parse(code: &str) -> IResult<&str, If> {
    map(
        tuple((
            tag("if"),
            space1,
            rvalue::parse,
            space0,
            compound::parse,
            opt(map(
                tuple((parsing::in_multispace(tag("else")), compound::parse)),
                |(_, else_content)| else_content,
            )),
        )),
        |(_, _, condition, _, content, else_content)| If {
            condition,
            content,
            else_content,
        },
    )(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expression::VariableRef;

    #[test]
    fn can_parse() {
        let if_statement = parse("if a { let b: i32 = 1; }").unwrap().1;
        assert_eq!(if_statement.condition, VariableRef("a".to_string()).into());
        assert_eq!(if_statement.else_content, None);
        let if_statement = parse("if a { let b: i32 = 1; } else { let c: i32 = 2; }")
            .unwrap()
            .1;
        assert_eq!(if_statement.condition, VariableRef("a".to_string()).into());
        assert!(if_statement.else_content.is_some());
        assert!(parse("else { let d: i32 = 1; }").is_err());
        let (rest, if_statement) =
            parse("if a { let b: i32 = 1; } else { if d { let c: i32 = 2; } }").unwrap();
        assert!(rest.is_empty());
        assert_eq!(if_statement.condition, VariableRef("a".to_string()).into());
    }
}
