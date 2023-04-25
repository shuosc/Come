use std::fmt::Display;

use crate::ir::{function::FunctionDefinitionIndex, statement::IRStatement};

use super::IsAction;

#[derive(Debug, Clone)]
pub enum InsertPosition {
    Back(usize),
    Index(FunctionDefinitionIndex),
}

impl InsertPosition {
    pub fn unwrap_index(self) -> Option<FunctionDefinitionIndex> {
        if let InsertPosition::Index(index) = self {
            Some(index)
        } else {
            None
        }
    }
    pub fn as_index(&self) -> Option<&FunctionDefinitionIndex> {
        if let InsertPosition::Index(index) = self {
            Some(index)
        } else {
            None
        }
    }
}

impl Display for InsertPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InsertPosition::Back(block_index) => write!(f, "back of block {block_index}"),
            InsertPosition::Index(index) => write!(f, "({}, {})", index.0, index.1),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InsertStatement {
    pub position: InsertPosition,
    pub statement: IRStatement,
}

impl Display for InsertStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "insert `{}` at {}", self.statement, self.position)
    }
}

impl IsAction for InsertStatement {
    fn perform_on_function(self, ir: &mut crate::ir::FunctionDefinition) {
        match self.position {
            InsertPosition::Back(block_index) => {
                ir.content[block_index].content.push(self.statement);
            }
            InsertPosition::Index(index) => {
                ir.content[index.0].content.insert(index.1, self.statement);
            }
        }
    }
}

impl InsertStatement {
    pub fn at_index(
        index: impl Into<FunctionDefinitionIndex>,
        statement: impl Into<IRStatement>,
    ) -> Self {
        Self {
            position: InsertPosition::Index(index.into()),
            statement: statement.into(),
        }
    }
    pub fn front_of(block_index: usize, statement: impl Into<IRStatement>) -> Self {
        Self::at_index((block_index, 0usize), statement)
    }
    pub fn back_of(block_index: usize, statement: impl Into<IRStatement>) -> Self {
        Self {
            position: InsertPosition::Back(block_index),
            statement: statement.into(),
        }
    }
}
