use std::fmt;

use crate::{
    ir::{
        function::{GenerateRegister, HasRegister, UseRegister},
        quantity::{self, local, Quantity, RegisterName},
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
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Load {
    pub to: RegisterName,
    pub data_type: Type,
    pub from: Quantity,
}

impl HasRegister for Load {
    fn on_register_change(&mut self, from: &RegisterName, to: &Quantity) {
        if &self.to == from {
            self.to = to.clone().unwrap_local();
        }
    }
}

impl GenerateRegister for Load {
    fn generated_register(&self) -> Option<(RegisterName, Type)> {
        Some((self.to.clone(), self.data_type.clone()))
    }
}

impl UseRegister for Load {
    fn use_register(&self) -> Vec<RegisterName> {
        if let Quantity::RegisterName(register) = &self.from {
            vec![register.clone()]
        } else {
            Vec::new()
        }
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

    #[test]
    fn test_parse() {
        let result = parse("%0 = load i32 %1").unwrap().1;
        assert_eq!(
            result,
            Load {
                to: RegisterName("0".to_string()),
                data_type: data_type::I32.clone(),
                from: RegisterName("1".to_string()).into(),
            },
        );
    }
}
