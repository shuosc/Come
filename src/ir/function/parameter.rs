use nom::{
    bytes::complete::tag, character::complete::space0, combinator::map, sequence::tuple, IResult,
};

use crate::{
    ast,
    ir::{quantity::local, RegisterName},
    utility::data_type::{self, Type},
};

/// [`Parameter`] represents a function's parameter.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Parameter {
    /// Name of the parameter.
    pub name: RegisterName,
    /// Type of the parameter.
    pub data_type: Type,
}

pub fn parse(code: &str) -> IResult<&str, Parameter> {
    map(
        tuple((local::parse, space0, tag(":"), space0, data_type::parse)),
        |(name, _, _, _, data_type)| Parameter { name, data_type },
    )(code)
}

pub fn from_ast(ast: &ast::function_definition::Parameter) -> Parameter {
    let ast::function_definition::Parameter { name, data_type } = ast;
    Parameter {
        name: RegisterName(name.clone()),
        data_type: data_type.clone(),
    }
}
