use super::{statement, terminator};
use crate::{backend::riscv::FunctionCompileContext, ir};

/// Emit assembly code for a [`ir::function::basic_block::BasicBlock`].
pub fn emit_code(
    basic_block: &ir::function::basic_block::BasicBlock,
    ctx: &mut FunctionCompileContext,
) -> String {
    let ir::function::basic_block::BasicBlock {
        name,
        phis: _,
        content,
        terminator,
    } = basic_block;
    let mut result = String::new();
    if let Some(name) = name {
        result.push_str(format!("{}:\n", name).as_str());
    }
    for statement in content {
        let statement_code = statement::emit_code(statement, ctx);
        result.push_str(&statement_code);
    }
    if let Some(terminator) = terminator {
        let terminator_code = terminator::emit_code(terminator, ctx);
        result.push_str(&terminator_code);
    }
    result
}
