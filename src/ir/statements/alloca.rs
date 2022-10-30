use crate::{
    ir::{
        function::HasRegister,
        quantity::{local, Local},
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
use std::{
    collections::HashSet,
    fmt::{self, Display, Formatter},
};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Alloca {
    pub to: Local,
    pub alloc_type: Type,
}

impl HasRegister for Alloca {
    fn get_registers(&self) -> HashSet<Local> {
        let mut result = HashSet::new();
        result.insert(self.to.clone());
        result
    }
}

impl Display for Alloca {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} = alloca {}", self.to, self.alloc_type)
    }
}

pub fn parse(code: &str) -> IResult<&str, Alloca> {
    map(
        tuple((
            local::parse,
            space0,
            tag("="),
            space0,
            tag("alloca"),
            space1,
            data_type::parse,
        )),
        |(to_register, _, _, _, _, _, alloc_type)| Alloca {
            to: to_register,
            alloc_type,
        },
    )(code)
}
