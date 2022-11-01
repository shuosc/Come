use super::{
    basic_block::BasicBlock,
    statement::{Ret, Terminator},
};
use crate::ast::{self, statement::Statement};
use std::mem;

mod assign;
mod declare;
pub mod expression;
mod if_statement;
mod return_statement;
mod while_statement;
pub use expression::{lvalue_from_ast, rvalue_from_ast};

/// [`IRGeneratingContext`] is used to collect the basic blocks generated.
pub struct IRGeneratingContext<'a> {
    /// Parent [`crate::ir::IRGeneratingContext`]
    pub parent_context: &'a mut crate::ir::IRGeneratingContext,
    /// [`BasicBlock`]s that are already generated.
    pub done_basic_blocks: Vec<BasicBlock>,
    /// The [`BasicBlock`] that are in construction.
    pub current_basic_block: BasicBlock,
}

impl<'a> IRGeneratingContext<'a> {
    /// Create a new [`IRGeneratingContext`].
    pub fn new(parent_context: &'a mut crate::ir::IRGeneratingContext) -> Self {
        Self {
            parent_context,
            done_basic_blocks: Vec::new(),
            current_basic_block: BasicBlock::new(),
        }
    }

    /// Finish the current [`BasicBlock`] with `terminator` and start a new one.
    pub fn end_current_basic_block_with(&mut self, terminator: impl Into<Terminator>) {
        self.current_basic_block.terminator = Some(terminator.into());
        self.done_basic_blocks.push(mem::replace(
            &mut self.current_basic_block,
            BasicBlock::new(),
        ));
    }

    /// Finish generating [`BasicBlock`]s for the current function.
    /// Return the collected [`BasicBlock`]s.
    pub fn done(mut self) -> Vec<BasicBlock> {
        if !self.current_basic_block.empty() {
            if self.current_basic_block.terminator.is_none() {
                self.current_basic_block.terminator = Some(Ret { value: None }.into());
            }
            self.done_basic_blocks.push(self.current_basic_block);
        }
        self.done_basic_blocks
            .into_iter()
            .filter(|it| !it.empty())
            .collect()
    }
}

/// Generate IR from [`ast::statement::compound::Compound`].
pub fn compound_from_ast(ast: &ast::statement::compound::Compound, ctx: &mut IRGeneratingContext) {
    for statement in &ast.0 {
        match statement {
            Statement::Declare(declare) => declare::from_ast(declare, ctx),
            Statement::Assign(assign) => assign::from_ast(assign, ctx),
            Statement::Return(return_statement) => {
                return_statement::from_ast(return_statement, ctx)
            }
            Statement::If(if_statement) => if_statement::from_ast(if_statement, ctx),
            Statement::While(while_statement) => while_statement::from_ast(while_statement, ctx),
            Statement::FunctionCall(_) => todo!(),
        }
    }
}
