use crate::ir;

use super::FunctionCompileContext;

/// Compile a binary operator.
mod binary_calculate;
/// Compile a load command.
mod load;
/// Compile a store command.
mod store;
/// Compile a unary operator.
mod unary_calculate;

mod load_field;

mod branch;
mod ret;
mod set_field;

/// Emit assembly code for a [`ir::function::statement::IRStatement`].
pub fn emit_code(
    statement: &ir::function::statement::IRStatement,
    ctx: &mut FunctionCompileContext,
) -> String {
    match statement {
        ir::statement::IRStatement::Phi(_) => String::new(),
        ir::statement::IRStatement::Alloca(_) => String::new(),
        ir::statement::IRStatement::UnaryCalculate(unary_calculate) => {
            unary_calculate::emit_code(unary_calculate, ctx)
        }
        ir::statement::IRStatement::BinaryCalculate(binary_calculate) => {
            binary_calculate::emit_code(binary_calculate, ctx)
        }
        ir::statement::IRStatement::Load(load) => load::emit_code(load, ctx),
        ir::statement::IRStatement::Store(store) => store::emit_code(store, ctx),
        ir::statement::IRStatement::LoadField(load_field) => load_field::emit_code(load_field, ctx),
        ir::statement::IRStatement::SetField(set_field) => set_field::emit_code(set_field, ctx),
        ir::statement::IRStatement::Branch(branch) => branch::emit_code(branch, ctx),
        ir::statement::IRStatement::Jump(jump) => format!("    j {}\n", jump.label),
        ir::statement::IRStatement::Ret(ret) => ret::emit_code(ret, ctx),
    }
}
