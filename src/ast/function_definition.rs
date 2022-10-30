


use nom::{
    bytes::complete::tag,
    character::complete::{multispace0, space0},
    combinator::map,
    multi::separated_list0,
    sequence::{delimited, pair, tuple},
    IResult,
};

use crate::utility::{
    data_type::{self, Type},
    parsing,
};

use super::statement::compound::{self, Compound};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Parameter {
    pub name: String,
    pub data_type: Type,
}

pub fn parse_parameter(code: &str) -> IResult<&str, Parameter> {
    map(
        tuple((parsing::ident, space0, tag(":"), space0, data_type::parse)),
        |(name, _, _, _, data_type)| Parameter { name, data_type },
    )(code)
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct FunctionDefinition {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
    pub content: Compound,
}

pub fn parse(code: &str) -> IResult<&str, FunctionDefinition> {
    map(
        tuple((
            tag("fn"),
            space0,
            parsing::ident,
            delimited(
                pair(tag("("), space0),
                separated_list0(parsing::in_multispace(tag(",")), parse_parameter),
                pair(multispace0, tag(")")),
            ),
            parsing::in_multispace(tag("->")),
            data_type::parse,
            parsing::in_multispace(compound::parse),
        )),
        |(_, _, name, parameters, _, return_type, content)| FunctionDefinition {
            name,
            parameters,
            return_type,
            content,
        },
    )(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse() {
        let function_definition = parse(
            "fn add(a: i32, b: i32) -> i32 {
    return a + b;
}",
        )
        .unwrap()
        .1;
        assert_eq!(function_definition.name, "add");
        assert_eq!(function_definition.parameters.len(), 2);
    }
}
