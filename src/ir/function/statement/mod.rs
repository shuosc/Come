use std::fmt;

use enum_dispatch::enum_dispatch;
use nom::{branch::alt, combinator::map, IResult};
use paste::paste;

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

pub use alloca::Alloca;
pub use branch::Branch;
pub use calculate::{BinaryCalculate, UnaryCalculate};
pub use jump::Jump;
pub use load::Load;
pub use load_field::LoadField;
pub use phi::Phi;
pub use ret::Ret;
pub use set_field::SetField;
pub use store::Store;

use crate::{
    ir::{quantity::Quantity, RegisterName},
    utility::data_type::Type,
};

/// This trait should be implemented for all IRStatements
#[enum_dispatch]
pub trait IsIRStatement {
    fn use_register(&self) -> Vec<RegisterName>;
    fn generate_register(&self) -> Option<(RegisterName, Type)>;
    fn on_register_change(&mut self, from: &RegisterName, to: &Quantity);
}

/// A statement in a function.
#[enum_dispatch(IsIRStatement)]
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum IRStatement {
    Phi,
    Alloca,
    UnaryCalculate,
    BinaryCalculate,
    Load,
    Store,
    LoadField,
    SetField,
    Branch,
    Jump,
    Ret,
}

macro_rules! variant {
    ($name:ident, $variant:ident) => {
        paste! {
        impl IRStatement {
            /// Returns `Some(variant)` if the statement is this variant,
            /// return `None` if it is not.
            pub fn [<try_as_ $name>](&self) -> Option<&$variant> {
                match self {
                    IRStatement::$variant(inner) => Some(inner),
                    _ => None,
                }
            }

            /// Returns the variant if the statement is this variant,
            /// panic if it is not.
            pub fn [<as_ $name>](&self) -> &$variant {
                match self {
                    IRStatement::$variant(inner) => inner,
                    _ => panic!("Expected {} but got {:?}", stringify!($name), self),
                }
            }
        }
        }
    };
}

variant!(phi, Phi);
variant!(alloca, Alloca);
variant!(unary_calculate, UnaryCalculate);
variant!(binary_calculate, BinaryCalculate);
variant!(load, Load);
variant!(store, Store);
variant!(load_field, LoadField);
variant!(set_field, SetField);
variant!(branch, Branch);
variant!(jump, Jump);
variant!(ret, Ret);

/// Parse ir code to get a [`IRStatement`].
pub fn parse(code: &str) -> IResult<&str, IRStatement> {
    alt((
        map(phi::parse, IRStatement::Phi),
        map(alloca::parse, IRStatement::Alloca),
        map(calculate::unary::parse, IRStatement::UnaryCalculate),
        map(calculate::binary::parse, IRStatement::BinaryCalculate),
        map(load_field::parse, IRStatement::LoadField),
        map(load::parse, IRStatement::Load),
        map(store::parse, IRStatement::Store),
        map(branch::parse, IRStatement::Branch),
        map(jump::parse, IRStatement::Jump),
        map(ret::parse, IRStatement::Ret),
    ))(code)
}

impl fmt::Display for IRStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IRStatement::Phi(x) => x.fmt(f),
            IRStatement::Alloca(x) => x.fmt(f),
            IRStatement::UnaryCalculate(x) => x.fmt(f),
            IRStatement::BinaryCalculate(x) => x.fmt(f),
            IRStatement::Load(x) => x.fmt(f),
            IRStatement::Store(x) => x.fmt(f),
            IRStatement::LoadField(x) => x.fmt(f),
            IRStatement::SetField(x) => x.fmt(f),
            IRStatement::Branch(x) => x.fmt(f),
            IRStatement::Jump(x) => x.fmt(f),
            IRStatement::Ret(x) => x.fmt(f),
        }
    }
}
