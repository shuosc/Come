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

mod set_field;

/// Emit assembly code for a [`ir::function::statement::IRStatement`].
pub fn emit_code(
    statement: &ir::function::statement::ContentStatement,
    ctx: &mut FunctionCompileContext,
) -> String {
    match statement {
        ir::statement::ContentStatement::Alloca(_) => String::new(),
        ir::statement::ContentStatement::UnaryCalculate(unary_calculate) => {
            unary_calculate::emit_code(unary_calculate, ctx)
        }
        ir::statement::ContentStatement::BinaryCalculate(binary_calculate) => {
            binary_calculate::emit_code(binary_calculate, ctx)
        }
        ir::statement::ContentStatement::Load(load) => load::emit_code(load, ctx),
        ir::statement::ContentStatement::Store(store) => store::emit_code(store, ctx),
        ir::statement::ContentStatement::LoadField(load_field) => load_field::emit_code(load_field, ctx),
        ir::statement::ContentStatement::SetField(set_field) => set_field::emit_code(set_field, ctx),
    }
}
