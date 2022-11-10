use crate::{
    ir::{function::GenerateRegister, RegisterName},
    utility::{data_type::Type, parsing},
};
use nom::{
    bytes::complete::tag, character::complete::space1, combinator::map, sequence::tuple, IResult,
};
use std::{
    fmt,
    fmt::{Display, Formatter},
};

/// [`Jump`] instruction.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Jump {
    pub label: String,
}

impl GenerateRegister for Jump {
    fn register(&self) -> Option<(RegisterName, Type)> {
        None
    }
}

impl Display for Jump {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "j {}", self.label)
    }
}

/// Parse ir code to get a [`Jump`] instruction.
pub fn parse(code: &str) -> IResult<&str, Jump> {
    map(
        tuple((tag("j"), space1, parsing::ident)),
        |(_, _, label)| Jump { label },
    )(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let result = parse("j label").unwrap().1;
        assert_eq!(
            result,
            Jump {
                label: "label".to_string(),
            },
        );
    }
}
