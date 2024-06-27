use crate::{
    ir::{
        function::IsIRStatement,
        quantity::{self, Quantity},
        RegisterName,
    },
    utility::data_type::Type,
};
use nom::{
    bytes::complete::tag,
    character::complete::space0,
    combinator::{map, opt},
    sequence::tuple,
    IResult,
};
use serde::{Deserialize, Serialize};
use std::fmt;
/// [`Ret`] instruction.
#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct Ret {
    pub value: Option<Quantity>,
}

impl IsIRStatement for Ret {
    fn on_register_change(&mut self, from: &RegisterName, to: Quantity) {
        if let Some(Quantity::RegisterName(local)) = &mut self.value {
            if local == from {
                *local = to.unwrap_local();
            }
        }
    }
    fn generate_register(&self) -> Option<(RegisterName, Type)> {
        None
    }
    fn use_register(&self) -> Vec<RegisterName> {
        if let Some(Quantity::RegisterName(register)) = &self.value {
            vec![register.clone()]
        } else {
            Vec::new()
        }
    }
}

impl fmt::Display for Ret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(value) = &self.value {
            write!(f, "ret {value}")
        } else {
            write!(f, "ret")
        }
    }
}

/// Parse a [`Ret`] instruction.
pub fn parse(code: &str) -> IResult<&str, Ret> {
    map(
        tuple((tag("ret"), space0, opt(quantity::parse))),
        |(_, _, value)| Ret { value },
    )(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let result = parse("ret").unwrap().1;
        assert_eq!(result, Ret { value: None });
        let result = parse("ret %1").unwrap().1;
        assert_eq!(
            result,
            Ret {
                value: Some(RegisterName("1".to_string()).into())
            }
        )
    }
}
