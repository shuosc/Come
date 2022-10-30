use crate::ast::{expression, expression::function_call};
use nom::{bytes::complete::tag, combinator::map, sequence::pair, IResult};

/// [`FunctionCall`] represents a standalone function call.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct FunctionCall(pub expression::function_call::FunctionCall);

/// Parse source code to get an [`FunctionCall`].
pub fn parse(code: &str) -> IResult<&str, FunctionCall> {
    map(pair(function_call::parse, tag(";")), |(content, _)| {
        FunctionCall(content)
    })(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse() {
        let assign = parse("a();").unwrap().1;
        assert_eq!(assign.0.name, "a".to_string());
        assert_eq!(assign.0.arguments.len(), 0);

        let assign = parse("a(1, 2);").unwrap().1;
        assert_eq!(assign.0.name, "a".to_string());
        assert_eq!(assign.0.arguments.len(), 2);
    }
}
