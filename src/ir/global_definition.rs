use std::fmt;

use super::IRGeneratingContext;
use crate::{
    ast::{self, expression::rvalue::RValue},
    ir::{
        integer_literal,
        integer_literal::IntegerLiteral,
        quantity::{global, GlobalVariableName},
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
use serde::{Deserialize, Serialize};

/// [`GlobalDefinition`] represents a global variable definition.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct GlobalDefinition {
    /// Name of the global variable.
    pub name: GlobalVariableName,
    /// Type of the global variable.
    pub data_type: Type,
    // todo: Other literals
    /// Initial value of the global variable.
    pub initial_value: IntegerLiteral,
}

impl fmt::Display for GlobalDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "@{}: {} = {}",
            self.name, self.data_type, self.initial_value
        )
    }
}

/// Parse ir code to get a [`GlobalDefinition`].
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
            name: item,
            data_type,
            initial_value,
        },
    )(code)
}

/// Generate ir code from a [`GlobalDefinition`].
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
        name: GlobalVariableName(variable_name.clone()),
        data_type: data_type.clone(),
        initial_value,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::borrow_interior_mutable_const)]
    use super::*;

    #[test]
    fn can_parse() {
        let code = "@g = global i32 100";
        let result = parse(code).unwrap().1;
        assert_eq!(
            result,
            GlobalDefinition {
                name: GlobalVariableName("g".to_string()),
                data_type: data_type::I32.clone(),
                initial_value: IntegerLiteral(100),
            }
        );
    }
}
