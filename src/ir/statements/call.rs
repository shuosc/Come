use crate::{
    ir::{
        function::HasRegister,
        quantity::{local, local_or_number_literal, Local, LocalOrNumberLiteral},
    },
    utility::{data_type, data_type::Type, parsing},
};
use nom::{
    bytes::complete::tag,
    character::complete::space0,
    combinator::{map, opt},
    multi::separated_list0,
    sequence::{delimited, tuple},
    IResult,
};
use std::{
    collections::HashSet,
    fmt::{self, Display, Formatter},
};

pub struct Call {
    to: Option<Local>,
    data_type: Type,
    name: String,
    params: Vec<LocalOrNumberLiteral>,
}

impl HasRegister for Call {
    fn get_registers(&self) -> HashSet<Local> {
        let mut result = HashSet::new();
        if let Some(to) = &self.to {
            result.insert(to.clone());
        }
        result
    }
}

impl Display for Call {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(to_register) = &self.to {
            write!(f, "{} = ", to_register)?;
        }
        write!(f, "call {} {}(", self.data_type, self.name)?;
        write!(
            f,
            "{}",
            self.params
                .iter()
                .map(|it| format!("{}", it))
                .collect::<Vec<_>>()
                .join(",")
        )?;
        write!(f, ")")
    }
}

pub fn parse(code: &str) -> IResult<&str, Call> {
    map(
        tuple((
            opt(map(tuple((local::parse, space0, tag("="), space0)), |x| {
                x.0
            })),
            tag("call"),
            space0,
            data_type::parse,
            space0,
            parsing::ident,
            delimited(
                tag("("),
                separated_list0(tuple((space0, tag(","), space0)), local_or_number_literal),
                tag(")"),
            ),
        )),
        |(result, _, _, data_type, _, name, params)| Call {
            to: result,
            data_type,
            name,
            params,
        },
    )(code)
}
