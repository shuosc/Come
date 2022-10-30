use std::fmt;

use crate::{
    ir::{
        function::HasRegister,
        quantity::{local_or_number_literal, LocalOrNumberLiteral},
        Local,
    },
    utility::{data_type, data_type::Type, parsing},
};
use nom::{
    bytes::complete::tag,
    character::complete::{space0, space1},
    combinator::map,
    multi::many1,
    sequence::{preceded, tuple},
    IResult,
};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Field {
    pub name: String,
    pub index: Vec<usize>,
}

pub fn parse_field(code: &str) -> IResult<&str, Field> {
    map(
        tuple((parsing::ident, many1(preceded(tag("."), parsing::integer)))),
        |(name, index)| Field {
            name,
            index: index.into_iter().map(|i| i as usize).collect(),
        },
    )(code)
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct SetField {
    pub data_type: Type,
    pub value: LocalOrNumberLiteral,
    pub field: Field,
}

impl HasRegister for SetField {
    fn get_registers(&self) -> std::collections::HashSet<Local> {
        std::collections::HashSet::new()
    }
}

impl fmt::Display for SetField {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

pub fn parse(code: &str) -> IResult<&str, SetField> {
    map(
        tuple((
            tag("setfield"),
            space1,
            data_type::parse,
            space1,
            parse_field,
            space0,
            tag(","),
            space0,
            local_or_number_literal,
        )),
        |(_, _, data_type, _, field, _, _, _, value)| SetField {
            data_type,
            field,
            value,
        },
    )(code)
}
