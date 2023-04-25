use std::fmt::Display;

use enum_dispatch::enum_dispatch;

use crate::ir::FunctionDefinition;
mod insert_basic_block;
mod insert_statement;
mod remove_statement;
mod rename_local;
pub use insert_basic_block::InsertBasicBlock;
pub use insert_statement::InsertStatement;
pub use remove_statement::RemoveStatement;
pub use rename_local::RenameLocal;

#[enum_dispatch]
pub trait IsAction {
    fn perform_on_function(self, ir: &mut FunctionDefinition);
}

#[enum_dispatch(IsAction)]
#[derive(Debug, Clone)]
pub enum Action {
    InsertStatement,
    RemoveStatement,
    RenameLocal,
    InsertBasicBlock,
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::InsertStatement(action) => write!(f, "{action}"),
            Action::RemoveStatement(action) => write!(f, "{action}"),
            Action::RenameLocal(action) => write!(f, "{action}"),
            Action::InsertBasicBlock(action) => write!(f, "{action}"),
        }
    }
}
