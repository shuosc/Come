use enum_dispatch::enum_dispatch;

use super::{
    analyze,
};


mod remove_unused_register;
pub use remove_unused_register::RemoveUnusedRegister;

#[enum_dispatch]
pub trait Pass {
    fn run<'a>(
        &self,
        ir: &'a mut super::FunctionDefinition,
        analyzer: &analyze::FunctionAnalyzer,
    );
}

#[enum_dispatch(Pass)]
pub enum Passes {
    RemoveUnusedRegister,
}

pub struct EliminateSingleStore {}

pub struct MemoryToRegister {}

pub struct Optimizer {
    passes: Vec<Passes>,
}
