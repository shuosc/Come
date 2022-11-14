mod remove_only_once_store;
mod remove_unused_register;
use super::editor::IRFunctionEditor;
use enum_dispatch::enum_dispatch;

use remove_unused_register::RemoveUnusedRegister;
use remove_only_once_store::RemoveOnlyOnceStore;

#[enum_dispatch]
pub trait IsPass {
    fn run<'a>(&self, ir: &mut IRFunctionEditor);
}

#[enum_dispatch(IsPass)]
pub enum Pass {
    RemoveUnusedRegister,
    RemoveOnlyOnceStore,
}
