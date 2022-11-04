use std::fmt;

use crate::{
    ir::{
        function::GenerateRegister,
        quantity::{self, local, Quantity},
        LocalVariableName,
    },
    utility::{data_type, data_type::Type, parsing},
};
use nom::{
    bytes::complete::tag,
    character::complete::{space0, space1},
    combinator::map,
    multi::many1,
    sequence::{preceded, tuple},
    IResult,
};

/// Reference to a field.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Field {
    /// Name of the field.
    pub name: Quantity,
    /// Index of the field.
    pub index: Vec<usize>,
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        for index in &self.index {
            write!(f, ".{}", index)?;
        }
        Ok(())
    }
}

/// Parse ir code to get a field reference.
pub fn parse_field(code: &str) -> IResult<&str, Field> {
    map(
        tuple((quantity::parse, many1(preceded(tag("."), parsing::integer)))),
        |(name, index)| Field {
            name,
            index: index.into_iter().map(|i| i as usize).collect(),
        },
    )(code)
}

/// [`SetField`] instruction.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct SetField {
    /// Result register
    pub result: LocalVariableName,
    /// Type of the value to store.
    pub data_type: Type,
    /// Value to store.
    pub value: Quantity,
    /// Where to store the value.
    pub field: Field,
}

impl GenerateRegister for SetField {
    fn register(&self) -> Option<LocalVariableName> {
        None
    }
}

impl fmt::Display for SetField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} = setfield {} {}, {}",
            self.result, self.data_type, self.value, self.field
        )
    }
}

/// Parse ir code to get a [`SetField`] instruction.
pub fn parse(code: &str) -> IResult<&str, SetField> {
    map(
        tuple((
            local::parse,
            space0,
            tag("="),
            space0,
            tag("setfield"),
            space1,
            data_type::parse,
            space1,
            parse_field,
            space0,
            tag(","),
            space0,
            quantity::parse,
        )),
        |(result, _, _, _, _, _, data_type, _, field, _, _, _, value)| SetField {
            data_type,
            field,
            value,
            result,
        },
    )(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utility::data_type::Integer;

    #[test]
    fn test_parse() {
        let code = "%2 = setfield i32 %0.0.1, %1";
        let (_, set_field) = parse(code).unwrap();
        assert_eq!(
            set_field,
            SetField {
                data_type: Type::Integer(Integer {
                    signed: true,
                    width: 32
                }),
                field: Field {
                    name: LocalVariableName("0".to_string()).into(),
                    index: vec![0, 1],
                },
                value: LocalVariableName("1".to_string()).into(),
                result: LocalVariableName("2".to_string())
            }
        );
    }
}
