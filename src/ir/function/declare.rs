use crate::{
    ast,
    ir::{quantity::local::Local, statements::IRStatement},
};

use super::IRGeneratingContext;
use crate::ir::statements::Alloca;
pub fn from_ast(ast: &ast::statement::declare::Declare, ctx: &mut IRGeneratingContext) {
    let ast::statement::declare::Declare {
        variable_name,
        data_type,
        init_value,
    } = ast;
    ctx.current_basic_block
        .content
        .push(IRStatement::Alloca(Alloca {
            to: Local(format!("{}_addr", variable_name)),
            alloc_type: data_type.clone(),
        }));
    if let Some(init_value) = init_value {
        let assign_statement = ast::statement::assign::Assign {
            lhs: ast::expression::lvalue::LValue::VariableRef(
                ast::expression::variable_ref::VariableRef(variable_name.clone()),
            ),
            rhs: init_value.clone(),
        };
        super::assign::from_ast(&assign_statement, ctx);
    }
}
