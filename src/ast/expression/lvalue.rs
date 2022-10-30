use enum_dispatch::enum_dispatch;
use nom::{branch::alt, combinator::map, IResult};

use super::{
    field_access::{self, FieldAccess},
    variable_ref::{self, VariableRef},
};

#[enum_dispatch]
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum LValue {
    VariableRef,
    FieldAccess,
}

pub fn parse(code: &str) -> IResult<&str, LValue> {
    alt((
        map(field_access::parse, LValue::FieldAccess),
        map(variable_ref::parse, LValue::VariableRef),
    ))(code)
}

// todo: test
