use crate::{
    ast::{self, expression::rvalue::RValue},
    ir::{
        integer_literal,
        integer_literal::IntegerLiteral,
        quantity::{global, Global},
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

use super::IRGeneratingContext;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct GlobalDefinition {
    pub item: Global,
    pub data_type: Type,
    // todo: Other literals
    pub initial_value: IntegerLiteral,
}

pub fn parse(code: &str) -> IResult<&str, GlobalDefinition> {
    map(
        tuple((
            global::parse,
            space0,
            tag("="),
            space0,
            tag("global"),
            space1,
            data_type::parse,
            space1,
            integer_literal::parse,
        )),
        |(item, _, _, _, _, _, data_type, _, initial_value)| GlobalDefinition {
            item,
            data_type,
            initial_value,
        },
    )(code)
}

pub fn from_ast(
    ast: &crate::ast::global_definition::VariableDefinition,
    _ctx: &mut IRGeneratingContext,
) -> GlobalDefinition {
    let ast::statement::declare::Declare {
        variable_name,
        data_type,
        init_value,
    } = &ast.0;
    let initial_value = if let Some(RValue::IntegerLiteral(initial_value)) = init_value {
        initial_value.clone().into()
    } else if init_value.is_none() {
        IntegerLiteral(0)
    } else {
        unimplemented!()
    };

    GlobalDefinition {
        item: Global(variable_name.clone()),
        data_type: data_type.clone(),
        initial_value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse() {
        let code = "@g = global i32 100";
        let result = parse(code).unwrap().1;
        println!("{:?}", result);
    }
}
