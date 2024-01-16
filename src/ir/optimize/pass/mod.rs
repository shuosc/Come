mod fix_irreducible;
mod fix_irreducible_new;
mod memory_to_register;
mod remove_load_directly_after_store;
mod remove_only_once_store;
mod remove_unused_register;
mod topological_sort;
use crate::ir::editor::Editor;
use enum_dispatch::enum_dispatch;
pub use fix_irreducible::FixIrreducible;
use memory_to_register::MemoryToRegister;
use remove_load_directly_after_store::RemoveLoadDirectlyAfterStore;
use remove_only_once_store::RemoveOnlyOnceStore;
use remove_unused_register::RemoveUnusedRegister;
use std::str::FromStr;
pub use topological_sort::TopologicalSort;
/// This trait should be implemented by all passes which can do optimizing on ir function.
#[enum_dispatch]
pub trait IsPass {
    fn run(&self, editor: &mut Editor);

    /// Which passes this pass requires to be executed before it.
    fn need(&self) -> Vec<Pass>;

    /// Which passes this pass will invalidate.
    fn invalidate(&self) -> Vec<Pass>;
}

/// All passes which can do optimizing on ir function.
#[enum_dispatch(IsPass)]
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Pass {
    RemoveUnusedRegister,
    RemoveOnlyOnceStore,
    RemoveLoadDirectlyAfterStore,
    MemoryToRegister,
    FixIrreducible,
    TopologicalSort,
}

impl FromStr for Pass {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "RemoveUnusedRegister" => Ok(Self::RemoveUnusedRegister(RemoveUnusedRegister)),
            "RemoveOnlyOnceStore" => Ok(Self::RemoveOnlyOnceStore(RemoveOnlyOnceStore)),
            "RemoveLoadDirectlyAfterStore" => Ok(Self::RemoveLoadDirectlyAfterStore(
                RemoveLoadDirectlyAfterStore,
            )),
            "MemoryToRegister" => Ok(Self::MemoryToRegister(MemoryToRegister)),
            "FixIrreducible" => Ok(Self::FixIrreducible(FixIrreducible)),
            "TopologicalSort" => Ok(Self::TopologicalSort(TopologicalSort)),
            _ => Err(()),
        }
    }
}

impl From<&str> for Pass {
    fn from(s: &str) -> Self {
        Self::from_str(s).unwrap()
    }
}
