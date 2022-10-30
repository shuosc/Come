use std::collections::HashMap;

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

use super::IRGeneratingContext;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TypeDefinition {
    pub name: String,
    pub fields: Vec<Type>,
}

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
                tuple((multispace0, tag("{"), multispace0)),
                separated_list0(
                    tuple((multispace0, tag(","), multispace0)),
                    data_type::parse,
                ),
                tuple((multispace0, tag("}"), multispace0)),
            ),
        )),
        |(_, name, _, _, _, _, _, fields)| TypeDefinition { name, fields },
    )(code)
}

pub struct TypeDefinitionMapping {
    pub field_names: HashMap<String, usize>,
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse() {
        let code = "%S = type {\n
    i32,\n
    i32\n
}";
        let result = parse(code).unwrap().1;
        println!("{:?}", result);
    }
}
