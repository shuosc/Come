use std::fmt;

use crate::{
    ir::{
        function::GenerateRegister,
        quantity::{self, local, LocalVariableName, Quantity},
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

/// [`Load`] instruction.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Load {
    pub to: LocalVariableName,
    pub data_type: Type,
    pub from: Quantity,
}

impl GenerateRegister for Load {
    fn register(&self) -> Option<LocalVariableName> {
        Some(self.to.clone())
    }
}

impl fmt::Display for Load {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = load {} {}", self.to, self.data_type, self.from)
    }
}

/// Parse ir code to get a [`Load`] instruction.
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
            quantity::parse,
        )),
        |(to, _, _, _, _, _, data_type, _, from)| Load {
            to,
            data_type,
            from,
        },
    )(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utility::data_type::Integer;

    #[test]
    fn test_parse() {
        let result = parse("%0 = load i32 %1").unwrap().1;
        assert_eq!(
            result,
            Load {
                to: LocalVariableName("0".to_string()),
                data_type: Type::Integer(Integer {
                    width: 32,
                    signed: true,
                }),
                from: LocalVariableName("1".to_string()).into(),
            },
        );
    }
}
