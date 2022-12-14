use super::action::EditActionBatch;
use crate::ir::analyzer::Analyzer;

mod memory_to_register;
mod remove_load_directly_after_store;
mod remove_only_once_store;
mod remove_unused_register;

use enum_dispatch::enum_dispatch;
use memory_to_register::MemoryToRegister;
use remove_load_directly_after_store::RemoveLoadDirectlyAfterStore;
use remove_only_once_store::RemoveOnlyOnceStore;
use remove_unused_register::RemoveUnusedRegister;

#[enum_dispatch]
pub trait IsPass {
    fn run(&self, analyzer: &Analyzer) -> EditActionBatch;
}

#[enum_dispatch(IsPass)]
pub enum Pass {
    RemoveUnusedRegister,
    RemoveOnlyOnceStore,
    RemoveLoadDirectlyAfterStore,
    MemoryToRegister,
}
