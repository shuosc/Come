mod remove_load_directly_after_store;
mod remove_only_once_store;
mod remove_unused_register;
use super::editor::IRFunctionEditor;
use enum_dispatch::enum_dispatch;

use remove_load_directly_after_store::RemoveLoadDirectlyAfterStore;
use remove_only_once_store::RemoveOnlyOnceStore;
use remove_unused_register::RemoveUnusedRegister;

#[enum_dispatch]
pub trait IsPass {
    fn run<'a>(&self, editor: &mut IRFunctionEditor);
}

#[enum_dispatch(IsPass)]
pub enum Pass {
    RemoveUnusedRegister,
    RemoveOnlyOnceStore,
    RemoveLoadDirectlyAfterStore,
}
