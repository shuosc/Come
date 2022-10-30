use crate::ir::statements::{Ret, Terminator};

use super::*;

pub fn from_ast(ast: &ast::statement::return_statement::Return, ctx: &mut IRGeneratingContext) {
    let ast::statement::return_statement::Return(value) = ast;
    let return_value = value
        .clone()
        .map(|value| expression::rvalue_from_ast(&value, ctx));
    ctx.current_basic_block.terminator = Some(Terminator::Ret(Ret {
        value: return_value,
    }));
}
