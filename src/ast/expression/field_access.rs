use nom::{
    branch::alt, bytes::complete::tag, combinator::map, multi::fold_many0, sequence::preceded,
    IResult,
};

use crate::utility::parsing::{self, ident};

use super::{function_call, in_brackets, integer_literal, rvalue::RValue, variable_ref};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct FieldAccess {
    from: Box<RValue>,
    name: String,
}

pub(in crate::ast::expression) fn higher_than_field_access(code: &str) -> IResult<&str, RValue> {
    alt((
        map(variable_ref::parse, RValue::VariableRef),
        map(in_brackets::parse, RValue::InBrackets),
        map(function_call::parse, RValue::FunctionCall),
        map(integer_literal::parse, RValue::IntegerLiteral),
    ))(code)
}

pub fn parse(code: &str) -> IResult<&str, FieldAccess> {
    let (rest, first) = higher_than_field_access(code)?;
    let (rest, second) = preceded(tag("."), ident)(rest)?;
    fold_many0(
        preceded(parsing::in_multispace(tag(".")), ident),
        move || FieldAccess {
            from: Box::new(first.clone()),
            name: second.clone(),
        },
        |acc, next| FieldAccess {
            from: Box::new(RValue::FieldAccess(acc)),
            name: next,
        },
    )(rest)
}

// todo: test
