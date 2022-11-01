use nom::{combinator::map, IResult};

use super::statement::declare::{self, Declare};

/// [`VariableDefinition`] represents a global variable definition.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct VariableDefinition(pub Declare);

/// Parse source code to get a [`VariableDefinition`].
pub fn parse(code: &str) -> IResult<&str, VariableDefinition> {
    map(declare::parse, VariableDefinition)(code)
}
