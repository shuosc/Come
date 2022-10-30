use crate::utility::parsing;
use nom::{combinator::map, IResult};

/// [`VariableRef`] represents we are referring a variable.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct VariableRef(pub String);

/// Parse source code to get a [`VariableRef`].
pub fn parse(code: &str) -> IResult<&str, VariableRef> {
    map(parsing::ident, VariableRef)(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse() {
        let variable_ref = parse("a").unwrap().1;
        assert_eq!(variable_ref.0, "a");
        let variable_ref = parse("a123").unwrap().1;
        assert_eq!(variable_ref.0, "a123");
        let variable_ref = parse("a_123").unwrap().1;
        assert_eq!(variable_ref.0, "a_123");
    }
}
