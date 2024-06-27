use crate::{
    ir::{
        function::IsIRStatement,
        quantity::{local, Quantity, RegisterName},
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
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
/// [`Alloca`] instruction.
#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct Alloca {
    /// Local variable, pointing to the space allocated on the stack.
    pub to: RegisterName,
    /// Type of the space allocated on the stack.
    pub alloc_type: Type,
}

impl IsIRStatement for Alloca {
    fn on_register_change(&mut self, from: &RegisterName, to: Quantity) {
        if &self.to == from {
            self.to = to.unwrap_local();
        }
    }
    fn generate_register(&self) -> Option<(RegisterName, Type)> {
        Some((self.to.clone(), Type::Address))
    }
    fn use_register(&self) -> Vec<RegisterName> {
        Vec::new()
    }
}

impl Display for Alloca {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} = alloca {}", self.to, self.alloc_type)
    }
}

/// Parse ir code to get an [`Alloca`] instruction.
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

#[cfg(test)]
pub mod test_util {
    #![allow(clippy::borrow_interior_mutable_const)]

    use super::*;

    pub fn new(variable_name: &str) -> Alloca {
        Alloca {
            to: RegisterName(format!("{variable_name}_addr")),
            alloc_type: data_type::I32.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::borrow_interior_mutable_const)]
    use super::*;

    #[test]
    fn test_parse() {
        let result = parse("%0 = alloca i32").unwrap().1;
        assert_eq!(
            result,
            Alloca {
                to: RegisterName("0".to_string()),
                alloc_type: data_type::I32.clone()
            }
        );
    }
}
