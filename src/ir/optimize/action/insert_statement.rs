use crate::ir::{function::FunctionDefinitionIndex, statement::IRStatement};

use super::{remove_statement::RemoveStatement, Action, IsAction};

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

pub struct InsertStatement {
    pub position: InsertPosition,
    pub statement: IRStatement,
}

impl IsAction for InsertStatement {
    fn perform(self, ir: &mut crate::ir::FunctionDefinition) {
        match self.position {
            InsertPosition::Back(block_index) => {
                ir.content[block_index].content.push(self.statement);
            }
            InsertPosition::Index(index) => {
                ir.content[index.0].content.insert(index.1, self.statement);
            }
        }
    }

    fn affect_others<'a>(&self, others: impl Iterator<Item = &'a mut Action>) {
        if let Some(self_index) = self.position.as_index() {
            for other in others {
                match other {
                    Action::InsertStatement(InsertStatement {
                        position: InsertPosition::Index(other_index),
                        ..
                    }) if other_index.0 == self_index.0 && other_index.1 >= self_index.1 => {
                        other_index.1 += 1;
                    }
                    Action::RemoveStatement(RemoveStatement { index })
                        if index.0 == self_index.0 && index.1 >= self_index.1 =>
                    {
                        index.1 += 1;
                    }
                    Action::InsertStatement(_) => (),
                    Action::RemoveStatement(_) => (),
                    Action::RenameLocal(_) => (),
                }
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
