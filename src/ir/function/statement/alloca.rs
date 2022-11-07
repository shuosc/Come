use crate::{
    ir::{
        function::GenerateRegister,
        quantity::{local, LocalVariableName},
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
use std::fmt::{self, Display, Formatter};

/// [`Alloca`] instruction.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Alloca {
    /// Local variable, pointing to the space allocated on the stack.
    pub to: LocalVariableName,
    /// Type of the space allocated on the stack.
    pub alloc_type: Type,
}

impl GenerateRegister for Alloca {
    fn register(&self) -> Option<(LocalVariableName, Type)> {
        Some((self.to.clone(), Type::Address))
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
mod tests {
    use super::*;
    use crate::utility::data_type::Integer;

    #[test]
    fn test_parse() {
        let result = parse("%0 = alloca i32").unwrap().1;
        assert_eq!(
            result,
            Alloca {
                to: LocalVariableName("0".to_string()),
                alloc_type: Type::Integer(Integer {
                    signed: true,
                    width: 32
                })
            }
        );
    }
}
