use crate::{
    ir::{
        function::GenerateRegister,
        quantity::{local, LocalVariableName},
    },
    utility::{data_type, data_type::Type, parsing},
};
use nom::{
    bytes::complete::tag,
    character::complete::{space0, space1},
    combinator::map,
    sequence::tuple,
    IResult,
};
use std::fmt;

/// [`LoadField`] instruction.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct LoadField {
    /// Where to store the result of the load.
    pub to: LocalVariableName,
    /// Where to load from.
    pub source: LocalVariableName,
    /// Type of the field to load.
    pub data_type: Type,
    /// Offset of the field to load.
    pub index: usize,
}

impl fmt::Display for LoadField {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl GenerateRegister for LoadField {
    fn register(&self) -> Option<LocalVariableName> {
        Some(self.to.clone())
    }
}

/// Parse ir code to get a [`LoadField`] instruction.
pub fn parse(code: &str) -> IResult<&str, LoadField> {
    map(
        tuple((
            local::parse,
            space0,
            tag("="),
            space0,
            tag("loadfield"),
            space1,
            data_type::parse,
            space1,
            local::parse,
            space0,
            tag(","),
            space0,
            parsing::integer,
        )),
        |(to, _, _, _, _, _, data_type, _, source, _, _, _, index)| LoadField {
            to,
            data_type,
            source,
            index: index as _,
        },
    )(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utility::data_type::Integer;

    #[test]
    fn test_parse() {
        let result = parse("%1 = loadfield i32 %0, 0").unwrap().1;
        assert_eq!(
            result,
            LoadField {
                to: LocalVariableName("1".to_string()),
                data_type: Type::Integer(Integer {
                    signed: true,
                    width: 32,
                }),
                source: LocalVariableName("0".to_string()),
                index: 0,
            },
        );
    }
}
