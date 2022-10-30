use std::fmt;

use crate::{
    ir::{
        function::HasRegister,
        quantity::{local, local_or_global, Local, LocalOrGlobal},
    },
    utility::{data_type, data_type::Type},
};
use nom::{
    bytes::complete::tag,
    character::complete::{space0, space1},
    combinator::map,
    sequence::tuple,
    IResult,
};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Load {
    pub to: Local,
    pub data_type: Type,
    pub from: LocalOrGlobal,
}

impl HasRegister for Load {
    fn get_registers(&self) -> std::collections::HashSet<Local> {
        let mut result = std::collections::HashSet::new();
        result.insert(self.to.clone());
        result
    }
}

impl fmt::Display for Load {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = load {} {}", self.to, self.data_type, self.from)
    }
}

pub fn parse(code: &str) -> IResult<&str, Load> {
    map(
        tuple((
            local::parse,
            space0,
            tag("="),
            space0,
            tag("load"),
            space1,
            data_type::parse,
            space1,
            local_or_global,
        )),
        |(to, _, _, _, _, _, data_type, _, from)| Load {
            to,
            data_type,
            from,
        },
    )(code)
}
