use std::fmt;

use crate::{
    ir::quantity::{local, Local},
    utility::{
        data_type,
        data_type::Type,
        parsing::{self, in_multispace},
    },
};
use nom::{
    bytes::complete::tag,
    character::complete::space0,
    combinator::map,
    multi::separated_list1,
    sequence::{delimited, tuple},
    IResult,
};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PhiSource {
    name: Local,
    block: String,
}

fn parse_phi_source(code: &str) -> IResult<&str, PhiSource> {
    map(
        delimited(
            tag("["),
            tuple((local::parse, space0, tag(","), space0, parsing::ident)),
            tag("]"),
        ),
        |(name, _, _, _, block)| PhiSource { name, block },
    )(code)
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Phi {
    pub to: Local,
    data_type: Type,
    from: Vec<PhiSource>,
}

impl fmt::Display for Phi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = phi {}", self.to, self.data_type)?;
        for (i, source) in self.from.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "[{}, {}]", source.name, source.block)?;
        }
        Ok(())
    }
}

pub fn parse(code: &str) -> IResult<&str, Phi> {
    map(
        tuple((
            local::parse,
            space0,
            tag("="),
            space0,
            data_type::parse,
            separated_list1(in_multispace(tag(",")), in_multispace(parse_phi_source)),
        )),
        |(to, _, _, _, data_type, from)| Phi {
            to,
            data_type,
            from,
        },
    )(code)
}
