use std::fmt::Display;

use super::IsAction;

#[derive(Debug, Clone)]
pub struct RemoveBasicBlock {
    pub index: usize,
}

impl RemoveBasicBlock {
    pub fn new(index: impl Into<usize>) -> Self {
        Self {
            index: index.into(),
        }
    }
}

impl Display for RemoveBasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "remove basic block at {}", self.index)
    }
}

impl IsAction for RemoveBasicBlock {
    fn perform_on_function(self, ir: &mut crate::ir::FunctionDefinition) {
        ir.content.remove(self.index);
    }
}
