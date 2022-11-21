use crate::{
    ir::{
        function::IsIRStatement,
        quantity::{self, Quantity},
        RegisterName,
    },
    utility::{data_type, data_type::Type},
};
use nom::{
    bytes::complete::tag,
    character::{complete::space1, streaming::space0},
    combinator::map,
    sequence::tuple,
    IResult,
};
use std::fmt;

/// [`Store`] instruction.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Store {
    /// Type of the value to store.
    pub data_type: Type,
    /// Value to store.
    pub source: Quantity,
    /// Where to store the value.
    pub target: Quantity,
}

impl IsIRStatement for Store {
    fn on_register_change(&mut self, from: &RegisterName, to: Quantity) {
        if let Quantity::RegisterName(local) = &mut self.source {
            if local == from {
                *local = to.clone().unwrap_local();
            }
        }
        if let Quantity::RegisterName(local) = &mut self.target {
            if local == from {
                *local = to.unwrap_local();
            }
        }
    }
    fn generate_register(&self) -> Option<(RegisterName, Type)> {
        None
    }
    fn use_register(&self) -> Vec<RegisterName> {
        let mut result = Vec::new();
        if let Quantity::RegisterName(register) = &self.source {
            result.push(register.clone());
        }
        if let Quantity::RegisterName(register) = &self.target {
            result.push(register.clone());
        }
        result
    }
}

impl fmt::Display for Store {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "store {} {}, address {}",
            self.data_type, self.source, self.target
        )
    }
}

/// Parse ir code to get a [`Store`] instruction.
pub fn parse(code: &str) -> IResult<&str, Store> {
    map(
        tuple((
            tag("store"),
            space1,
            data_type::parse,
            space1,
            quantity::parse,
            space0,
            tag(","),
            space0,
            tag("address"),
            space1,
            quantity::parse,
        )),
        |(_, _, data_type, _, source, _, _, _, _, _, target)| Store {
            data_type,
            source,
            target,
        },
    )(code)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::borrow_interior_mutable_const)]

    use super::*;

    #[test]
    fn test_parse() {
        let code = "store i32 %0, address %1";
        let (_, store) = parse(code).unwrap();
        assert_eq!(
            store,
            Store {
                data_type: data_type::I32.clone(),
                source: RegisterName("0".to_string()).into(),
                target: RegisterName("1".to_string()).into(),
            }
        );
    }
}
