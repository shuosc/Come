use super::IRGeneratingContext;
use crate::{
    ast,
    utility::{data_type, data_type::Type, parsing},
};
use nom::{
    bytes::complete::tag,
    character::complete::multispace0,
    combinator::map,
    multi::separated_list0,
    sequence::{delimited, tuple},
    IResult,
};
use std::collections::HashMap;

/// [`TypeDefinition`] represents definition of a struct.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TypeDefinition {
    pub name: String,
    pub fields: Vec<Type>,
}

/// Parse ir code to get a [`TypeDefinition`].
pub fn parse(code: &str) -> IResult<&str, TypeDefinition> {
    map(
        tuple((
            tag("%"),
            parsing::ident,
            multispace0,
            tag("="),
            multispace0,
            tag("type"),
            multispace0,
            delimited(
                parsing::in_multispace(tag("{")),
                separated_list0(parsing::in_multispace(tag(",")), data_type::parse),
                parsing::in_multispace(tag("}")),
            ),
        )),
        |(_, name, _, _, _, _, _, fields)| TypeDefinition { name, fields },
    )(code)
}

/// Map field name to its index.
pub struct TypeDefinitionMapping {
    pub field_names: HashMap<String, usize>,
}

/// Generate ir code from a [`TypeDefinition`].
pub fn from_ast(
    ast: &ast::type_definition::TypeDefinition,
    ctx: &mut IRGeneratingContext,
) -> TypeDefinition {
    let ast::type_definition::TypeDefinition { name, fields } = ast;
    let mut field_names = HashMap::new();
    for (i, field) in ast.fields.iter().enumerate() {
        field_names.insert(field.name.clone(), i);
    }
    ctx.type_definitions
        .insert(name.clone(), TypeDefinitionMapping { field_names });
    TypeDefinition {
        name: name.clone(),
        fields: fields.iter().map(|field| field.data_type.clone()).collect(),
    }
}

// todo: tests