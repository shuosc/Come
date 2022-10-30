use super::{
    compound::{self, Compound},
    expression::rvalue::{self, RValue},
};
use nom::{
    bytes::complete::tag, character::complete::space0, combinator::map, sequence::tuple, IResult,
};

/// [`While`] represents a `while` statement.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct While {
    pub condition: RValue,
    pub content: Compound,
}

/// Parse source code to get a [`While`].
pub fn parse(code: &str) -> IResult<&str, While> {
    map(
        tuple((tag("while"), space0, rvalue::parse, space0, compound::parse)),
        |(_, _, condition, _, content)| While { condition, content },
    )(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expression::VariableRef;

    #[test]
    fn can_parse() {
        let while_statement = parse("while a { let b: i32 = 1; }").unwrap().1;
        assert_eq!(
            while_statement.condition,
            VariableRef("a".to_string()).into()
        );
    }
}
