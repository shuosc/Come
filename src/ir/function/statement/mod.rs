use std::fmt;

use enum_dispatch::enum_dispatch;
use nom::{branch::alt, combinator::map, IResult};
use paste::paste;
use serde::{Deserialize, Serialize};

/// Data structure, parser and ir generator for `alloca` statement.
mod alloca;
/// Data structure, parser and ir generator for `br` statement.
pub mod branch;
/// Data structure, parser and ir generator for calculations (unary or binary).
pub mod calculate;
/// Data structure, parser and ir generator for `call` statement.
pub mod call;
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

use crate::{
    ir::{quantity::Quantity, RegisterName},
    utility::data_type::Type,
};
pub use alloca::Alloca;
pub use branch::Branch;
pub use calculate::{BinaryCalculate, UnaryCalculate};
pub use call::Call;
pub use jump::Jump;
pub use load::Load;
pub use load_field::LoadField;
pub use phi::Phi;
pub use ret::Ret;
pub use set_field::SetField;
pub use store::Store;

/// This trait should be implemented for all IRStatements
#[enum_dispatch]
pub trait IsIRStatement {
    fn use_register(&self) -> Vec<RegisterName>;
    fn generate_register(&self) -> Option<(RegisterName, Type)>;
    fn on_register_change(&mut self, from: &RegisterName, to: Quantity);
}

/// A statement in a function.
#[enum_dispatch(IsIRStatement)]
#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub enum IRStatement {
    Phi,
    Alloca,
    Call,
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
            #[allow(dead_code)]
            pub fn [<try_as_ $name>](&self) -> Option<&$variant> {
                match self {
                    IRStatement::$variant(inner) => Some(inner),
                    _ => None,
                }
            }

            /// Returns the variant if the statement is this variant,
            /// panic if it is not.
            #[allow(dead_code)]
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
variant!(call, Call);
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
            IRStatement::Call(x) => x.fmt(f),
        }
    }
}

#[cfg(test)]
pub mod test_util {

    use crate::ir::function::basic_block::BasicBlock;

    use super::*;

    pub fn binop(target: &str, source1: &str, source2: &str) -> IRStatement {
        calculate::binary::test_util::new(target, source1, source2).into()
    }

    pub fn binop_constant(target: &str) -> IRStatement {
        calculate::binary::test_util::new_constant(target).into()
    }

    pub fn alloca(variable_name: &str) -> IRStatement {
        alloca::test_util::new(variable_name).into()
    }

    pub fn branch(target1: &str, target2: &str) -> IRStatement {
        branch::test_util::new(target1, target2).into()
    }

    pub fn jump(target: &str) -> IRStatement {
        jump::test_util::new(target).into()
    }

    pub fn load(variable_name: &str, to_id: usize) -> IRStatement {
        load::test_util::new(variable_name, to_id).into()
    }

    pub fn store(variable_name: &str) -> IRStatement {
        store::test_util::new(variable_name).into()
    }

    pub fn store_with_reg(variable_name: &str, reg: &str) -> IRStatement {
        store::test_util::with_reg_value(variable_name, reg).into()
    }

    pub fn phi(
        target: &str,
        source1_bb: &str,
        source1: &str,
        source2_bb: &str,
        source2: &str,
    ) -> IRStatement {
        phi::test_util::new(target, source1_bb, source1, source2_bb, source2).into()
    }

    pub fn jump_block(id: usize, to: usize) -> BasicBlock {
        BasicBlock {
            name: Some(format!("bb{}", id)),
            content: vec![jump(&format!("bb{}", to))],
        }
    }

    pub fn branch_block(id: usize, to1: usize, to2: usize) -> BasicBlock {
        BasicBlock {
            name: Some(format!("bb{}", id)),
            content: vec![branch(&format!("bb{}", to1), &format!("bb{}", to2))],
        }
    }

    pub fn ret_block(id: usize) -> BasicBlock {
        BasicBlock {
            name: Some(format!("bb{}", id)),
            content: vec![Ret { value: None }.into()],
        }
    }
}
