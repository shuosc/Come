use crate::{
    ir::{
        function::IsIRStatement,
        quantity::{self, local, Quantity, RegisterName},
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
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct PhiSource {
    pub value: Quantity,
    pub block: String,
}

impl PartialOrd for PhiSource {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.block.partial_cmp(&other.block)
    }
}

impl Ord for PhiSource {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.block.cmp(&other.block)
    }
}

fn parse_phi_source(code: &str) -> IResult<&str, PhiSource> {
    map(
        delimited(
            tag("["),
            tuple((quantity::parse, space0, tag(","), space0, parsing::ident)),
            tag("]"),
        ),
        |(name, _, _, _, block)| PhiSource { value: name, block },
    )(code)
}

/// [`Phi`] instruction.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Phi {
    /// Where to store the result of the phi.
    pub to: RegisterName,
    /// Type of the phi.
    pub data_type: Type,
    /// Sources of the phi.
    pub from: Vec<PhiSource>,
}

impl IsIRStatement for Phi {
    fn on_register_change(&mut self, from: &RegisterName, to: Quantity) {
        if &self.to == from {
            self.to = to.clone().unwrap_local();
        }
        for source in &mut self.from {
            if let Quantity::RegisterName(local) = &mut source.value {
                if local == from {
                    *local = to.clone().unwrap_local();
                }
            }
        }
    }
    fn generate_register(&self) -> Option<(RegisterName, Type)> {
        Some((self.to.clone(), self.data_type.clone()))
    }

    fn use_register(&self) -> Vec<RegisterName> {
        self.from
            .iter()
            .filter_map(|PhiSource { value: name, .. }| name.as_local())
            .cloned()
            .collect()
    }
}

impl fmt::Display for Phi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = phi {} ", self.to, self.data_type)?;
        for (i, source) in self.from.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "[{}, {}]", source.block, source.value)?;
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
pub mod test_util {
    #![allow(clippy::borrow_interior_mutable_const)]

    use super::*;
    pub fn new(
        target: &str,
        source1_bb: &str,
        source1: &str,
        source2_bb: &str,
        source2: &str,
    ) -> Phi {
        Phi {
            to: RegisterName(target.to_string()),
            data_type: data_type::I32.clone(),
            from: vec![
                PhiSource {
                    value: RegisterName(source1.to_string()).into(),
                    block: source1_bb.to_string(),
                },
                PhiSource {
                    value: RegisterName(source2.to_string()).into(),
                    block: source2_bb.to_string(),
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::borrow_interior_mutable_const)]
    use super::*;

    #[test]
    fn test_parse() {
        let result = parse("%1 = phi i32 [%2, bb1], [%4, bb2]").unwrap().1;
        assert_eq!(
            result,
            Phi {
                to: RegisterName("1".to_string()),
                data_type: data_type::I32.clone(),
                from: vec![
                    PhiSource {
                        value: RegisterName("2".to_string()).into(),
                        block: "bb1".to_string(),
                    },
                    PhiSource {
                        value: RegisterName("4".to_string()).into(),
                        block: "bb2".to_string(),
                    },
                ],
            }
        );
    }
}
