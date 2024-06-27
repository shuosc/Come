use std::fmt;

use crate::{
    ir::{
        function::IsIRStatement,
        quantity::{self, local, Quantity},
        RegisterName,
    },
    utility::{
        data_type,
        data_type::Type,
        parsing::{self, in_multispace},
    },
};
use nom::{
    bytes::complete::tag,
    character::complete::space1,
    combinator::map,
    multi::separated_list1,
    sequence::{delimited, tuple},
    IResult,
};
use serde::{Deserialize, Serialize};
/// [`SetField`] instruction.
#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct SetField {
    /// Where to store the result.
    pub target: RegisterName,
    /// What value to set.
    pub source: Quantity,
    /// Which value to set.
    pub origin_root: RegisterName,
    /// Access `.0`th field of the struct, which is `.1` type.
    pub field_chain: Vec<(Type, usize)>,
    /// `source`'s type.
    pub final_type: Type,
}

impl IsIRStatement for SetField {
    fn on_register_change(&mut self, from: &RegisterName, to: Quantity) {
        if &self.target == from {
            self.target = to.clone().unwrap_local();
        }
        if let Quantity::RegisterName(local) = &mut self.source {
            if local == from {
                *local = to.clone().unwrap_local();
            }
        }
        if &self.origin_root == from {
            self.origin_root = to.unwrap_local();
        }
    }
    fn generate_register(&self) -> Option<(RegisterName, Type)> {
        Some((self.target.clone(), self.field_chain[0].0.clone()))
    }
    fn use_register(&self) -> Vec<RegisterName> {
        if let Quantity::RegisterName(register) = &self.source {
            vec![register.clone()]
        } else {
            vec![]
        }
    }
}

impl fmt::Display for SetField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} = setfield {} {}.[{}] {}",
            self.target,
            self.final_type,
            self.origin_root,
            self.field_chain
                .iter()
                .map(|(t, i)| format!("{t}.{i}"))
                .collect::<Vec<_>>()
                .join(", "),
            self.source
        )
    }
}

fn parse_field(code: &str) -> IResult<&str, (Type, usize)> {
    map(
        tuple((data_type::parse, tag("."), parsing::integer)),
        |(t, _, i)| (t, i),
    )(code)
}

/// Parse ir code to get a [`SetField`] instruction.
pub fn parse(code: &str) -> IResult<&str, SetField> {
    map(
        tuple((
            local::parse,
            space1,
            tag("="),
            space1,
            tag("setfield"),
            space1,
            data_type::parse,
            space1,
            local::parse,
            tag("."),
            delimited(
                tag("["),
                separated_list1(tag(","), in_multispace(parse_field)),
                tag("]"),
            ),
            space1,
            quantity::parse,
        )),
        |(
            target,
            _,
            _eq,
            _,
            _setfield,
            _,
            final_type,
            _,
            origin_root,
            _dot,
            field_chain,
            _,
            source,
        )| SetField {
            target,
            source,
            origin_root,
            field_chain,
            final_type,
        },
    )(code)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::borrow_interior_mutable_const)]

    use super::*;

    #[test]
    fn test_parse() {
        let code = "%2 = setfield i32 %1.[SS.1, S.0] %0";
        let (_, set_field) = parse(code).unwrap();
        assert_eq!(
            set_field,
            SetField {
                source: RegisterName("0".to_string()).into(),
                origin_root: RegisterName("1".to_string()),
                field_chain: vec![
                    (Type::StructRef("SS".to_string()), 1),
                    (Type::StructRef("S".to_string()), 0),
                ],
                final_type: data_type::I32.clone(),
                target: RegisterName("2".to_string())
            }
        );
    }
}
