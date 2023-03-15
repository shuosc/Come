use crate::ir::function::FunctionDefinitionIndex;

use super::{
    insert_statement::{InsertPosition, InsertStatement},
    Action, IsAction,
};

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

impl IsAction for RemoveStatement {
    fn perform(self, ir: &mut crate::ir::FunctionDefinition) {
        ir.content[self.index.0].content.remove(self.index.1);
    }

    fn affect_others<'a>(&self, others: impl Iterator<Item = &'a mut Action>) {
        for other in others {
            match other {
                // todo: reconsider this, index.1 != 0 may not be here
                Action::InsertStatement(InsertStatement {
                    position: InsertPosition::Index(index),
                    ..
                }) if index.0 == self.index.0 && index.1 >= self.index.1 && index.1 != 0 => {
                    index.1 -= 1;
                }
                Action::RemoveStatement(other)
                    if other.index.0 == self.index.0
                        && other.index.1 >= self.index.1
                        && other.index.1 != 0 =>
                {
                    other.index.1 -= 1;
                }
                Action::InsertStatement(_) => (),
                Action::RemoveStatement(_) => (),
                Action::RenameLocal(_) => (),
            }
        }
    }
}
