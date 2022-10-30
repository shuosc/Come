use nom::{
    bytes::complete::tag,
    combinator::map,
    multi::separated_list0,
    sequence::{delimited, tuple},
    IResult,
};

use crate::utility::parsing;

use super::rvalue::{self, RValue};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Vec<RValue>,
}

pub(in crate::ast) fn parse(code: &str) -> IResult<&str, FunctionCall> {
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
        // todo: more cases
        let function_call = parse("f()").unwrap().1;
        assert_eq!(function_call.name, "f");
        let function_call = parse("f(a,b)").unwrap().1;
        assert_eq!(function_call.name, "f");
    }
}
