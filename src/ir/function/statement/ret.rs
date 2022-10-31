use crate::ir::{
    function::GenerateRegister,
    quantity::{self, Quantity},
    LocalVariableName,
};
use nom::{
    bytes::complete::tag,
    character::complete::space0,
    combinator::{map, opt},
    sequence::tuple,
    IResult,
};
use std::fmt;

/// [`Ret`] instruction.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Ret {
    pub value: Option<Quantity>,
}

impl GenerateRegister for Ret {
    fn register(&self) -> Option<LocalVariableName> {
        None
    }
}

impl fmt::Display for Ret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(value) = &self.value {
            write!(f, "ret {}", value)
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
                value: Some(LocalVariableName("1".to_string()).into())
            }
        )
    }
}
