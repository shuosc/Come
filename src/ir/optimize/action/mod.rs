use std::{collections::VecDeque, fmt::Display};

use enum_dispatch::enum_dispatch;

use crate::ir::FunctionDefinition;
mod insert_statement;
mod remove_statement;
mod rename_local;
pub use insert_statement::InsertStatement;
pub use remove_statement::RemoveStatement;
pub use rename_local::RenameLocal;

#[enum_dispatch]
pub trait IsAction {
    fn perform(self, ir: &mut FunctionDefinition);
    fn affect_others<'a>(&self, others: impl Iterator<Item = &'a mut Action>);
}

#[enum_dispatch(IsAction)]
#[derive(Debug, Clone)]
pub enum Action {
    InsertStatement,
    RemoveStatement,
    RenameLocal,
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::InsertStatement(action) => write!(f, "{action}"),
            Action::RemoveStatement(action) => write!(f, "{action}"),
            Action::RenameLocal(action) => write!(f, "{action}"),
        }
    }
}

#[derive(Default)]
pub struct Actions(VecDeque<Action>);

impl Actions {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }
    pub fn push(&mut self, action: impl Into<Action>) {
        self.0.push_back(action.into());
    }
    pub fn merge(mut self, mut other: Self) -> Self {
        self.0.append(&mut other.0);
        self
    }
    pub fn perform(self, ir: &mut FunctionDefinition) {
        let mut actions = self.0;
        while let Some(action) = actions.pop_front() {
            action.affect_others(actions.iter_mut());
            action.perform(ir);
        }
    }
}

impl FromIterator<Action> for Actions {
    fn from_iter<T: IntoIterator<Item = Action>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}
