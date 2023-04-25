use std::fmt::Display;

use crate::ir::function::basic_block::BasicBlock;

use super::IsAction;

#[derive(Debug, Clone)]
pub enum InsertPosition {
    Back,
    Index(usize),
}

impl Display for InsertPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InsertPosition::Back => write!(f, "back of function"),
            InsertPosition::Index(index) => write!(f, "{index}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InsertBasicBlock {
    pub position: InsertPosition,
    pub name: String,
}

impl IsAction for InsertBasicBlock {
    fn perform_on_function(self, ir: &mut crate::ir::FunctionDefinition) {
        match self.position {
            InsertPosition::Back => {
                ir.content.push(BasicBlock::new(self.name));
            }
            InsertPosition::Index(index) => {
                ir.content.insert(index, BasicBlock::new(self.name));
            }
        }
    }
}

impl Display for InsertBasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "insert `{}` at {}", self.name, self.position)
    }
}

impl InsertBasicBlock {
    pub fn at_index(index: impl Into<usize>, name: String) -> Self {
        Self {
            position: InsertPosition::Index(index.into()),
            name,
        }
    }
    pub fn back_of(name: String) -> Self {
        Self {
            position: InsertPosition::Back,
            name,
        }
    }
}
