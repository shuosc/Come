use std::fmt;

use enum_dispatch::enum_dispatch;
use nom::{branch::alt, combinator::map, IResult};

/// Data structure, parser and ir generator for `alloca` statement.
mod alloca;
/// Data structure, parser and ir generator for `br` statement.
pub mod branch;
/// Data structure, parser and ir generator for calculations (unary or binary).
pub mod calculate;
/// Data structure, parser and ir generator for `call` statement.
mod call;
/// Data structure, parser and ir generator for `j` statement.
mod jump;
/// Data structure, parser and ir generator for `load` statement.
mod load;
/// Data structure, parser and ir generator for `loadfield` statement.
mod load_field;
/// Data structure, parser and ir generator for `phi` statement.
pub mod phi;
/// Data structure, parser and ir generator for `ret` statement.
mod ret;
/// Data structure, parser and ir generator for `setfield` statement.
mod set_field;
/// Data structure, parser and ir generator for `store` statement.
mod store;

pub use alloca::Alloca;
pub use branch::Branch;
pub use calculate::{BinaryCalculate, UnaryCalculate};
pub use jump::Jump;
pub use load::Load;
pub use load_field::LoadField;
pub use ret::Ret;
pub use set_field::SetField;
pub use store::Store;

/// A statement in a function.
#[enum_dispatch(GenerateRegister)]
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

/// Parse ir code to get a [`IRStatement`].
pub fn parse_ir_statement(code: &str) -> IResult<&str, IRStatement> {
    alt((
        map(alloca::parse, IRStatement::Alloca),
        map(calculate::unary::parse, IRStatement::UnaryCalculate),
        map(calculate::binary::parse, IRStatement::BinaryCalculate),
        map(load_field::parse, IRStatement::LoadField),
        map(load::parse, IRStatement::Load),
        map(store::parse, IRStatement::Store),
    ))(code)
}

/// A special instruction that must exists at the end of a basic block.
#[enum_dispatch(GenerateRegister)]
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

/// Parse ir code to get a [`Terminator`] instruction.
pub fn parse_terminator(code: &str) -> IResult<&str, Terminator> {
    alt((
        map(branch::parse, Terminator::Branch),
        map(jump::parse, Terminator::Jump),
        map(ret::parse, Terminator::Ret),
    ))(code)
}

// todo: test
