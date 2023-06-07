use std::mem;

use self::{
    action::{
        InsertBasicBlock, InsertStatement, IsAction, RemoveBasicBlock, RemoveStatement, RenameLocal,
    },
    analyzer::{BindedAnalyzer, IsAnalyzer},
};
use crate::ir::function::basic_block::BasicBlock;

use super::{
    function::{formalize, FunctionDefinitionIndex},
    quantity::Quantity,
    statement::IRStatement,
    RegisterName,
};

mod action;
/// Analyzers for providing information of an ir function.
pub mod analyzer;
pub use analyzer::Analyzer;
pub struct Editor {
    // todo: remove this pub
    pub content: super::FunctionDefinition,
    pub analyzer: analyzer::Analyzer,
}

impl Editor {
    pub fn new(content: super::FunctionDefinition) -> Self {
        Self {
            content: formalize(content),
            analyzer: analyzer::Analyzer::new(),
        }
    }

    pub fn insert_statement(
        &mut self,
        index: impl Into<FunctionDefinitionIndex>,
        statement: impl Into<IRStatement>,
    ) {
        self.perform_action(InsertStatement::at_index(index, statement));
    }

    pub fn push_front_statement(&mut self, index: usize, statement: impl Into<IRStatement>) {
        self.perform_action(InsertStatement::front_of(index, statement));
    }

    pub fn push_back_statement(&mut self, index: usize, statement: impl Into<IRStatement>) {
        self.perform_action(InsertStatement::back_of(index, statement));
    }

    pub fn remove_statement(&mut self, index: impl Into<FunctionDefinitionIndex>) {
        self.perform_action(RemoveStatement::new(index));
    }

    pub fn remove_statements<T: Into<FunctionDefinitionIndex> + Ord>(
        &mut self,
        indexes: impl IntoIterator<Item = T>,
    ) {
        let mut indexes = indexes.into_iter().collect::<Vec<_>>();
        indexes.sort();
        while let Some(index) = indexes.pop() {
            self.remove_statement(index);
        }
    }

    pub fn rename_local(&mut self, from: RegisterName, to: impl Into<Quantity>) {
        self.perform_action(RenameLocal::new(from, to));
    }

    fn perform_action(&mut self, action: impl Into<action::Action>) {
        let action = action.into();
        self.analyzer.on_action(&action);
        action.perform_on_function(&mut self.content);
    }

    pub fn binded_analyzer(&self) -> BindedAnalyzer {
        self.analyzer.bind(&self.content)
    }

    pub fn insert_basic_block(&mut self, name: String, index: impl Into<usize>) {
        self.perform_action(InsertBasicBlock::at_index(index, name));
    }

    pub fn create_basic_block(&mut self, name: String) -> usize {
        self.perform_action(InsertBasicBlock::back_of(name));
        self.content.content.len() - 1
    }

    pub fn remove_basic_block(&mut self, index: impl Into<usize>) -> BasicBlock {
        let index = index.into();
        let origin_content = mem::take(&mut self.content.content[index]);
        self.perform_action(RemoveBasicBlock::new(index));
        origin_content
    }

    pub fn replace_basic_block(
        &mut self,
        index: impl Into<usize>,
        block: BasicBlock,
    ) -> BasicBlock {
        let index = index.into();
        let removed_origin = self.remove_basic_block(index);
        self.perform_action(
            InsertBasicBlock::at_index(index, block.name.unwrap()).set_content(block.content),
        );
        removed_origin
    }
    pub fn swap_basic_block(&mut self, index0: impl Into<usize>, index1: impl Into<usize>) {
        let index0 = index0.into();
        let index1 = index1.into();
        if index1 < index0 {
            self.swap_basic_block(index1, index0)
        } else {
            let block0 = self.remove_basic_block(index0);
            let block1 = self.remove_basic_block(index1 - 1);
            self.perform_action(
                InsertBasicBlock::at_index(index0, block1.name.unwrap())
                    .set_content(block1.content),
            );
            self.perform_action(
                InsertBasicBlock::at_index(index1, block0.name.unwrap())
                    .set_content(block0.content),
            );
        }
    }
}
