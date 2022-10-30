use nom::{combinator::map, IResult};

use super::statement::declare::{self, Declare};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct VariableDefinition(pub Declare);

pub fn parse(code: &str) -> IResult<&str, VariableDefinition> {
    map(declare::parse, VariableDefinition)(code)
}
