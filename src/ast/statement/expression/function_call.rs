use super::rvalue::{self, RValue};
use crate::utility::parsing;
use nom::{
    bytes::complete::tag,
    combinator::map,
    multi::separated_list0,
    sequence::{delimited, tuple},
    IResult,
};

/// [`FunctionCall`] represents result of a function call.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct FunctionCall {
    /// Function name.
    pub name: String,
    /// Arguments used in the call.
    pub arguments: Vec<RValue>,
}

/// Parse source code to get a [`FunctionCall`].
pub fn parse(code: &str) -> IResult<&str, FunctionCall> {
    map(
        tuple((
            parsing::ident,
            delimited(
                tag("("),
                separated_list0(parsing::in_multispace(tag(",")), rvalue::parse),
                tag(")"),
            ),
        )),
        |(name, arguments)| FunctionCall { name, arguments },
    )(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse() {
        let function_call = parse("f()").unwrap().1;
        assert_eq!(function_call.name, "f");
        assert_eq!(function_call.arguments.len(), 0);
        let function_call = parse("f(a,b)").unwrap().1;
        assert_eq!(function_call.name, "f");
        assert_eq!(function_call.arguments.len(), 2);
    }
}
