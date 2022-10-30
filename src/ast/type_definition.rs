use nom::{
    bytes::complete::tag,
    character::complete::{multispace0, space0},
    combinator::map,
    multi::separated_list0,
    sequence::{delimited, tuple},
    IResult,
};
use crate::utility::{
    data_type::{self, Type},
    parsing,
};

/// [`FieldDefinition`] represents a struct's field.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct FieldDefinition {
    pub name: String,
    pub data_type: Type,
}

/// Parse source code to get a [`FieldDefinition`].
fn parse_field_definition(code: &str) -> IResult<&str, FieldDefinition> {
    map(
        tuple((
            multispace0,
            parsing::ident,
            space0,
            tag(":"),
            space0,
            data_type::parse,
        )),
        |(_, name, _, _, _, data_type)| FieldDefinition { name, data_type },
    )(code)
}

/// [`TypeDefinition`] represents a struct definition.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct TypeDefinition {
    pub name: String,
    pub fields: Vec<FieldDefinition>,
}

/// Parse source code to get a [`TypeDefinition`].
pub fn parse(code: &str) -> IResult<&str, TypeDefinition> {
    map(
        tuple((
            multispace0,
            tag("struct"),
            multispace0,
            parsing::ident,
            multispace0,
            delimited(
                tag("{"),
                separated_list0(
                    parsing::in_multispace(tag(",")),
                    parse_field_definition,
                ),
                parsing::in_multispace(tag("}")),
            ),
        )),
        |(_, _, _, name, _, fields)| TypeDefinition { name, fields },
    )(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse() {
        let type_definition = parse("struct Foo { a: i32, b: i32 }").unwrap().1;
        assert_eq!(type_definition.name, "Foo");
        assert_eq!(type_definition.fields.len(), 2);
        assert_eq!(type_definition.fields[0].name, "a");
        assert_eq!(type_definition.fields[1].name, "b");
    }
}