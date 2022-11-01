use super::expression::{lvalue, rvalue, LValue, RValue};
use crate::utility::parsing::in_multispace;
use nom::{
    bytes::complete::tag, character::complete::multispace0, combinator::map, sequence::tuple,
    IResult,
};

/// [`Assign`] represents an assign statement.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Assign {
    /// The left hand side of the assign statement.
    pub lhs: LValue,
    /// The right hand side of the assign statement.
    pub rhs: RValue,
}

/// Parse source code to get an [`Assign`].
pub fn parse(code: &str) -> IResult<&str, Assign> {
    map(
        tuple((
            lvalue::parse,
            in_multispace(tag("=")),
            rvalue::parse,
            multispace0,
            tag(";"),
        )),
        |(lhs, _, rhs, _, _)| Assign { lhs, rhs },
    )(code)
}

#[cfg(test)]
mod tests {
    use crate::ast::expression::{IntegerLiteral, VariableRef};

    use super::*;

    #[test]
    fn can_parse() {
        let assign = parse("a = 1;").unwrap().1;
        assert_eq!(assign.lhs, VariableRef("a".to_string()).into());
        assert_eq!(assign.rhs, IntegerLiteral(1).into());
        let assign = parse("a = b;").unwrap().1;
        assert_eq!(assign.lhs, VariableRef("a".to_string()).into());
        assert_eq!(assign.rhs, VariableRef("b".to_string()).into());
    }
}
