use crate::{
    ast,
    ir::{
        quantity::{local::Local, LocalOrNumberLiteral},
        statements::{calculate::UnaryOperation, IRStatement},
    },
    utility::data_type::{Integer, Type},
};

use super::super::IRGeneratingContext;
use crate::ir::statements::calculate::UnaryCalculate;

pub fn from_ast(
    ast: &ast::expression::unary_operator::UnaryOperatorResult,
    ctx: &mut IRGeneratingContext,
) -> LocalOrNumberLiteral {
    let ast::expression::unary_operator::UnaryOperatorResult { operator, operand } = ast;
    let result_register_id = ctx.parent_context.next_register_id;
    ctx.parent_context.next_register_id += 1;
    let rvalue_register = super::rvalue_from_ast(operand.as_ref(), ctx);
    match operator.as_str() {
        "+" => {
            ctx.parent_context.next_register_id -= 1;
            return rvalue_register;
        }
        "-" => ctx
            .current_basic_block
            .content
            .push(IRStatement::UnaryCalculate(UnaryCalculate {
                operation: UnaryOperation::Neg,
                operand: rvalue_register,
                to: Local(format!("{}", result_register_id)),
                data_type: Type::Integer(Integer {
                    signed: true,
                    width: 32,
                }),
            })),
        "!" => ctx
            .current_basic_block
            .content
            .push(IRStatement::UnaryCalculate(UnaryCalculate {
                operation: UnaryOperation::Not,
                operand: rvalue_register,
                to: Local(format!("{}", result_register_id)),
                data_type: Type::Integer(Integer {
                    signed: true,
                    width: 32,
                }),
            })),
        "~" => {
            unimplemented!()
        }
        _ => {
            unimplemented!()
        }
    }
    LocalOrNumberLiteral::Local(Local(format!("{}", result_register_id)))
}
