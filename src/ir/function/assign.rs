use crate::{
    ast,
    ir::statements,
    utility::data_type::{Integer, Type},
};

use super::IRGeneratingContext;

pub fn from_ast(ast: &ast::statement::assign::Assign, ctx: &mut IRGeneratingContext) {
    // rhs code
    let result_register = super::expression::rvalue_from_ast(&ast.rhs, ctx);
    // lhs code
    let lhs_address_register = super::expression::lvalue_from_ast(&ast.lhs, ctx);
    // store code
    ctx.current_basic_block
        .content
        .push(statements::IRStatement::Store(statements::Store {
            source: result_register,
            target: lhs_address_register,
            // todo: use real datatype
            data_type: Type::Integer(Integer {
                signed: true,
                width: 32,
            }),
        }));
}
