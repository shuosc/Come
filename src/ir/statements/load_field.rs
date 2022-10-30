use std::{collections::HashSet, fmt};

use crate::{
    ir::{
        function::HasRegister,
        quantity::{local, Local},
    },
    utility::{data_type, data_type::Type, parsing},
};
use nom::{
    bytes::complete::tag,
    character::complete::{space0, space1},
    combinator::map,
    sequence::tuple,
    IResult,
};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct LoadField {
    pub to: Local,
    pub data_type: Type,
    pub source: Local,
    pub index: usize,
}

impl fmt::Display for LoadField {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl HasRegister for LoadField {
    fn get_registers(&self) -> HashSet<Local> {
        let mut result = HashSet::new();
        result.insert(self.to.clone());
        result
    }
}

pub fn parse(code: &str) -> IResult<&str, LoadField> {
    map(
        tuple((
            local::parse,
            space0,
            tag("="),
            space0,
            tag("loadfield"),
            space1,
            data_type::parse,
            space1,
            local::parse,
            space0,
            tag(","),
            space0,
            parsing::integer,
        )),
        |(to, _, _, _, _, _, data_type, _, source, _, _, _, index)| LoadField {
            to,
            data_type,
            source,
            index: index as _,
        },
    )(code)
}
