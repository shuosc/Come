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
pub mod load_field;
/// Data structure, parser and ir generator for `phi` statement.
pub mod phi;
/// Data structure, parser and ir generator for `ret` statement.
mod ret;
/// Data structure, parser and ir generator for `setfield` statement.
pub(crate) mod set_field;
/// Data structure, parser and ir generator for `store` statement.
mod store;

pub use phi::Phi;
pub use alloca::Alloca;
pub use branch::Branch;
pub use calculate::{BinaryCalculate, UnaryCalculate};
pub use jump::Jump;
pub use load::Load;
pub use load_field::LoadField;
pub use ret::Ret;
pub use set_field::SetField;
pub use store::Store;

use crate::ir::RegisterName;

use super::{UseRegister, GenerateRegister};

/// A statement in a function.
#[enum_dispatch(GenerateRegister, UseRegister)]
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum ContentStatement {
    Alloca,
    UnaryCalculate,
    BinaryCalculate,
    Load,
    Store,
    LoadField,
    SetField,
}

impl fmt::Display for ContentStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentStatement::Alloca(x) => x.fmt(f),
            ContentStatement::UnaryCalculate(x) => x.fmt(f),
            ContentStatement::BinaryCalculate(x) => x.fmt(f),
            ContentStatement::Load(x) => x.fmt(f),
            ContentStatement::Store(x) => x.fmt(f),
            ContentStatement::LoadField(x) => x.fmt(f),
            ContentStatement::SetField(x) => x.fmt(f),
        }
    }
}

/// Parse ir code to get a [`IRStatement`].
pub fn parse_ir_statement(code: &str) -> IResult<&str, ContentStatement> {
    alt((
        map(alloca::parse, ContentStatement::Alloca),
        map(calculate::unary::parse, ContentStatement::UnaryCalculate),
        map(calculate::binary::parse, ContentStatement::BinaryCalculate),
        map(load_field::parse, ContentStatement::LoadField),
        map(load::parse, ContentStatement::Load),
        map(store::parse, ContentStatement::Store),
    ))(code)
}

/// A special instruction that must exists at the end of a basic block.
#[enum_dispatch(GenerateRegister, UseRegister)]
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
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

trait IsStatement {}

#[enum_dispatch(IsStatement)]
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum StatementRef<'a> {
    Phi(&'a Phi),
    Content(&'a ContentStatement),
    Terminator(&'a Terminator),
}

impl UseRegister for StatementRef<'_> {
    fn use_register(&self) -> Vec<RegisterName> {
        match self {
            StatementRef::Phi(x) => x.use_register(),
            StatementRef::Content(x) => x.use_register(),
            StatementRef::Terminator(x) => x.use_register(),
        }
    }
}

impl GenerateRegister for StatementRef<'_> {
    fn generated_register(&self) -> Option<(RegisterName,crate::utility::data_type::Type)> {
        match self {
            StatementRef::Phi(x) => x.generated_register(),
            StatementRef::Content(x) => x.generated_register(),
            StatementRef::Terminator(x) => x.generated_register(),
        }
    }
}

#[enum_dispatch(IsStatement)]
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum StatementRefMut<'a> {
    Phi(&'a mut Phi),
    Content(&'a mut ContentStatement),
    Terminator(&'a mut Terminator),
}

impl UseRegister for StatementRefMut<'_> {
    fn use_register(&self) -> Vec<RegisterName> {
        match self {
            StatementRefMut::Phi(x) => x.use_register(),
            StatementRefMut::Content(x) => x.use_register(),
            StatementRefMut::Terminator(x) => x.use_register(),
        }
    }
}

impl GenerateRegister for StatementRefMut<'_> {
    fn generated_register(&self) -> Option<(RegisterName,crate::utility::data_type::Type)> {
        match self {
            StatementRefMut::Phi(x) => x.generated_register(),
            StatementRefMut::Content(x) => x.generated_register(),
            StatementRefMut::Terminator(x) => x.generated_register(),
        }
    }
}

// todo: test
