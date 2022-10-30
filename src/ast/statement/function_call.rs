use crate::ast::{expression, expression::function_call};
use nom::{bytes::complete::tag, combinator::map, sequence::pair, IResult};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct FunctionCall(pub expression::function_call::FunctionCall);

pub fn parse(code: &str) -> IResult<&str, FunctionCall> {
    map(pair(function_call::parse, tag(";")), |(content, _)| {
        FunctionCall(content)
    })(code)
}
