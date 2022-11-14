use super::{statement, FunctionCompileContext};
use crate::ir;

/// Emit assembly code for a [`ir::function::basic_block::BasicBlock`].
pub fn emit_code(
    basic_block: &ir::function::basic_block::BasicBlock,
    ctx: &mut FunctionCompileContext,
) -> String {
    let ir::function::basic_block::BasicBlock { name, content } = basic_block;
    let mut result = String::new();
    if let Some(name) = name {
        result.push_str(format!("{}:\n", name).as_str());
    }
    for statement in content {
        let statement_code = statement::emit_code(statement, ctx);
        result.push_str(&statement_code);
    }
    result
}
