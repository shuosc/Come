use super::{
    field_access::{self, FieldAccess},
    rvalue::RValue,
    variable_ref::{self, VariableRef},
};
use enum_dispatch::enum_dispatch;
use nom::{branch::alt, combinator::map, IResult};

/// Tag trait for [`LValue`].
#[enum_dispatch]
trait IsLValue {}

/// [`LValue`] represents a value that can be assigned to.
#[enum_dispatch(IsLValue)]
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum LValue {
    VariableRef,
    FieldAccess,
}

/// Parse source code to get a [`LValue`].
pub fn parse(code: &str) -> IResult<&str, LValue> {
    alt((
        map(field_access::parse, LValue::FieldAccess),
        map(variable_ref::parse, LValue::VariableRef),
    ))(code)
}

impl TryFrom<RValue> for LValue {
    type Error = ();

    fn try_from(rvalue: RValue) -> Result<Self, Self::Error> {
        match rvalue {
            RValue::VariableRef(variable_ref) => Ok(LValue::VariableRef(variable_ref)),
            RValue::FieldAccess(field_access) => Ok(LValue::FieldAccess(field_access)),
            _ => Err(()),
        }
    }
}

// todo: test
