use std::fmt;

use enum_dispatch::enum_dispatch;
use nom::{branch::alt, combinator::map, IResult};

mod alloca;
pub mod branch;
pub mod calculate;
mod call;
mod jump;
mod load;
mod load_field;
pub mod phi;
mod ret;
mod set_field;
mod store;

use super::function::HasRegister;
use crate::ir::quantity::Local;
pub use alloca::Alloca;
pub use branch::Branch;
pub use calculate::{BinaryCalculate, UnaryCalculate};
pub use jump::Jump;
pub use load::Load;
pub use load_field::LoadField;
pub use ret::Ret;
pub use set_field::SetField;
use std::collections::HashSet;
pub use store::Store;

#[enum_dispatch(HasRegister)]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum IRStatement {
    Alloca,
    UnaryCalculate,
    BinaryCalculate,
    Load,
    Store,
    LoadField,
    SetField,
}

impl fmt::Display for IRStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IRStatement::Alloca(x) => x.fmt(f),
            IRStatement::UnaryCalculate(x) => x.fmt(f),
            IRStatement::BinaryCalculate(x) => x.fmt(f),
            IRStatement::Load(x) => x.fmt(f),
            IRStatement::Store(x) => x.fmt(f),
            IRStatement::LoadField(x) => x.fmt(f),
            IRStatement::SetField(x) => x.fmt(f),
        }
    }
}

pub fn parse_ir_statement(code: &str) -> IResult<&str, IRStatement> {
    alt((
        map(alloca::parse, IRStatement::Alloca),
        map(calculate::parse_unary, IRStatement::UnaryCalculate),
        map(calculate::parse_binary, IRStatement::BinaryCalculate),
        map(load_field::parse, IRStatement::LoadField),
        map(load::parse, IRStatement::Load),
        map(store::parse, IRStatement::Store),
    ))(code)
}

#[enum_dispatch]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Terminator {
    Branch,
    Jump,
    Ret,
}

impl fmt::Display for Terminator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Terminator::Branch(x) => x.fmt(f),
            Terminator::Jump(x) => x.fmt(f),
            Terminator::Ret(x) => x.fmt(f),
        }
    }
}

pub fn parse_terminator(code: &str) -> IResult<&str, Terminator> {
    alt((
        map(branch::parse, Terminator::Branch),
        map(jump::parse, Terminator::Jump),
        map(ret::parse, Terminator::Ret),
    ))(code)
}
