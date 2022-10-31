use crate::{
    ir::{
        function::GenerateRegister,
        quantity::{local, LocalVariableName},
    },
    utility::{
        data_type,
        data_type::Type,
        parsing::{self, in_multispace},
    },
};
use nom::{
    bytes::complete::tag,
    character::complete::space0,
    combinator::map,
    multi::separated_list1,
    sequence::{delimited, tuple},
    IResult,
};
use std::fmt;

/// [`Phi`]'s source.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PhiSource {
    pub name: LocalVariableName,
    pub block: String,
}

fn parse_phi_source(code: &str) -> IResult<&str, PhiSource> {
    map(
        delimited(
            tag("["),
            tuple((local::parse, space0, tag(","), space0, parsing::ident)),
            tag("]"),
        ),
        |(name, _, _, _, block)| PhiSource { name, block },
    )(code)
}

/// [`Phi`] instruction.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Phi {
    /// Where to store the result of the phi.
    pub to: LocalVariableName,
    /// Type of the phi.
    pub data_type: Type,
    /// Sources of the phi.
    pub from: Vec<PhiSource>,
}

impl GenerateRegister for Phi {
    fn register(&self) -> Option<LocalVariableName> {
        Some(self.to.clone())
    }
}

impl fmt::Display for Phi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = phi {}", self.to, self.data_type)?;
        for (i, source) in self.from.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "[{}, {}]", source.name, source.block)?;
        }
        Ok(())
    }
}

/// Parse ir code to get a [`Phi`] instruction.
pub fn parse(code: &str) -> IResult<&str, Phi> {
    map(
        tuple((
            local::parse,
            space0,
            tag("="),
            space0,
            tag("phi"),
            space0,
            data_type::parse,
            space0,
            separated_list1(in_multispace(tag(",")), in_multispace(parse_phi_source)),
        )),
        |(to, _, _, _, _, _, data_type, _, from)| Phi {
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
        let result = parse("%1 = phi i32 [%2, bb1], [%4, bb2]").unwrap().1;
        assert_eq!(
            result,
            Phi {
                to: LocalVariableName("1".to_string()),
                data_type: Type::Integer(Integer {
                    signed: true,
                    width: 32
                }),
                from: vec![
                    PhiSource {
                        name: LocalVariableName("2".to_string()),
                        block: "bb1".to_string(),
                    },
                    PhiSource {
                        name: LocalVariableName("4".to_string()),
                        block: "bb2".to_string(),
                    },
                ],
            }
        );
    }
}
