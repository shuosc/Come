pub mod global;
pub mod local;

pub use crate::ir::quantity::{global::GlobalVariableName, local::RegisterName};
use crate::utility::parsing;
use enum_dispatch::enum_dispatch;
use nom::{branch::alt, combinator::map, IResult};
use std::fmt::{self, Display, Formatter};

/// Tag trait for [`Quantity`].
#[enum_dispatch]
trait IsQuantity {}

/// [`Quantity`] represents a variable (global or local) or a constant value
#[enum_dispatch(IsQuantity)]
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Quantity {
    RegisterName,
    GlobalVariableName,
    NumberLiteral(i64),
}

impl Display for Quantity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Quantity::GlobalVariableName(global) => write!(f, "{}", global),
            Quantity::RegisterName(local) => write!(f, "{}", local),
            Quantity::NumberLiteral(number) => write!(f, "{}", number),
        }
    }
}

/// Parse source code to get a [`Quantity`].
pub fn parse(code: &str) -> IResult<&str, Quantity> {
    alt((
        map(local::parse, Quantity::RegisterName),
        map(global::parse, Quantity::GlobalVariableName),
        map(parsing::integer, Quantity::NumberLiteral),
    ))(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let result = parse("%foo").unwrap().1;
        assert_eq!(
            result,
            Quantity::RegisterName(RegisterName("foo".to_string()))
        );
        let result = parse("%0").unwrap().1;
        assert_eq!(
            result,
            Quantity::RegisterName(RegisterName("0".to_string()))
        );
        let result = parse("@foo").unwrap().1;
        assert_eq!(
            result,
            Quantity::GlobalVariableName(GlobalVariableName("foo".to_string()))
        );
        let result = parse("123").unwrap().1;
        assert_eq!(result, Quantity::NumberLiteral(123));
    }
}
