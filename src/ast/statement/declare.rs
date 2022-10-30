

use nom::{
    bytes::complete::tag,
    character::complete::{multispace0, space0, space1},
    combinator::{map, opt},
    sequence::tuple,
    IResult,
};

use crate::{
    ast::expression::rvalue::{self, RValue},
    utility::{
        data_type::{self, Type},
        parsing,
    },
};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Declare {
    pub variable_name: String,
    pub data_type: Type,
    pub init_value: Option<RValue>,
}

pub fn parse(code: &str) -> IResult<&str, Declare> {
    map(
        tuple((
            tag("let"),
            space1,
            parsing::ident,
            space0,
            tag(":"),
            space0,
            data_type::parse,
            space0,
            opt(map(
                tuple((space0, tag("="), multispace0, rvalue::parse, multispace0)),
                |(_, _, _, x, _)| x,
            )),
            tag(";"),
        )),
        |(_, _, variable_name, _, _, _, data_type, _, init_value, _)| Declare {
            variable_name,
            data_type,
            init_value,
        },
    )(code)
}
