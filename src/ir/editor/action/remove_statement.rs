use std::fmt::Display;

use crate::ir::function::FunctionDefinitionIndex;

use super::IsAction;

#[derive(Debug, Clone)]
pub struct RemoveStatement {
    pub index: FunctionDefinitionIndex,
}

impl RemoveStatement {
    pub fn new(index: impl Into<FunctionDefinitionIndex>) -> Self {
        Self {
            index: index.into(),
        }
    }
}

impl Display for RemoveStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "remove statement at ({}, {})",
            self.index.0, self.index.1
        )
    }
}

impl IsAction for RemoveStatement {
    fn perform_on_function(self, ir: &mut crate::ir::FunctionDefinition) {
        ir.content[self.index.0].content.remove(self.index.1);
    }
}
