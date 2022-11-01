use crate::{backend::riscv::FunctionCompileContext, ir::statement::Terminator};

/// Compile a branch command.
mod branch;
/// Compile a jump command.
mod ret;

/// Emit assembly code for a [`Terminator`].
pub fn emit_code(terminator: &Terminator, ctx: &mut FunctionCompileContext) -> String {
    match terminator {
        Terminator::Branch(branch) => branch::emit_code(branch, ctx),
        Terminator::Jump(jump) => format!("    j {}\n", jump.label),
        Terminator::Ret(ret) => ret::emit_code(ret, ctx),
    }
}
