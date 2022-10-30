use crate::{
    ast,
    ir::{
        quantity::{local::Local, LocalOrNumberLiteral},
        statements::{calculate::BinaryOperation, BinaryCalculate, IRStatement},
    },
    utility::data_type::{Integer, Type},
};

use super::IRGeneratingContext;

pub fn from_ast(
    ast: &ast::expression::binary_operator::BinaryOperatorResult,
    ctx: &mut IRGeneratingContext,
) -> LocalOrNumberLiteral {
    let ast::expression::binary_operator::BinaryOperatorResult { operator, lhs, rhs } = ast;
    let result_register_id = ctx.parent_context.next_register_id;
    ctx.parent_context.next_register_id += 1;
    let left_register = super::rvalue_from_ast(lhs.as_ref(), ctx);
    let right_register = super::rvalue_from_ast(rhs.as_ref(), ctx);
    match operator.as_str() {
        "+" => ctx
            .current_basic_block
            .content
            .push(IRStatement::BinaryCalculate(BinaryCalculate {
                operation: BinaryOperation::Add,
                operand1: left_register,
                operand2: right_register,
                to: Local(format!("{}", result_register_id)),
                data_type: Type::Integer(Integer {
                    signed: true,
                    width: 32,
                }),
            })),
        "-" => ctx
            .current_basic_block
            .content
            .push(IRStatement::BinaryCalculate(BinaryCalculate {
                operation: BinaryOperation::Sub,
                operand1: left_register,
                operand2: right_register,
                to: Local(format!("{}", result_register_id)),
                data_type: Type::Integer(Integer {
                    signed: true,
                    width: 32,
                }),
            })),
        "<<" => ctx
            .current_basic_block
            .content
            .push(IRStatement::BinaryCalculate(BinaryCalculate {
                operation: BinaryOperation::LogicalShiftLeft,
                operand1: left_register,
                operand2: right_register,
                to: Local(format!("{}", result_register_id)),
                data_type: Type::Integer(Integer {
                    signed: true,
                    width: 32,
                }),
            })),
        ">>" => ctx
            .current_basic_block
            .content
            .push(IRStatement::BinaryCalculate(BinaryCalculate {
                operation: BinaryOperation::LogicalShiftRight,
                operand1: left_register,
                operand2: right_register,
                to: Local(format!("{}", result_register_id)),
                data_type: Type::Integer(Integer {
                    signed: true,
                    width: 32,
                }),
            })),
        "==" => ctx
            .current_basic_block
            .content
            .push(IRStatement::BinaryCalculate(BinaryCalculate {
                operation: BinaryOperation::Equal,
                operand1: left_register,
                operand2: right_register,
                to: Local(format!("{}", result_register_id)),
                data_type: Type::Integer(Integer {
                    signed: true,
                    width: 32,
                }),
            })),
        "!=" => ctx
            .current_basic_block
            .content
            .push(IRStatement::BinaryCalculate(BinaryCalculate {
                operation: BinaryOperation::NotEqual,
                operand1: left_register,
                operand2: right_register,
                to: Local(format!("{}", result_register_id)),
                data_type: Type::Integer(Integer {
                    signed: true,
                    width: 32,
                }),
            })),
        "<" => ctx
            .current_basic_block
            .content
            .push(IRStatement::BinaryCalculate(BinaryCalculate {
                operation: BinaryOperation::LessThan,
                operand1: left_register,
                operand2: right_register,
                to: Local(format!("{}", result_register_id)),
                data_type: Type::Integer(Integer {
                    signed: true,
                    width: 32,
                }),
            })),
        "<=" => ctx
            .current_basic_block
            .content
            .push(IRStatement::BinaryCalculate(BinaryCalculate {
                operation: BinaryOperation::LessOrEqualThan,
                operand1: left_register,
                operand2: right_register,
                to: Local(format!("{}", result_register_id)),
                data_type: Type::Integer(Integer {
                    signed: true,
                    width: 32,
                }),
            })),
        ">" => ctx
            .current_basic_block
            .content
            .push(IRStatement::BinaryCalculate(BinaryCalculate {
                operation: BinaryOperation::GreaterThan,
                operand1: left_register,
                operand2: right_register,
                to: Local(format!("{}", result_register_id)),
                data_type: Type::Integer(Integer {
                    signed: true,
                    width: 32,
                }),
            })),
        ">=" => ctx
            .current_basic_block
            .content
            .push(IRStatement::BinaryCalculate(BinaryCalculate {
                operation: BinaryOperation::GreaterOrEqualThan,
                operand1: left_register,
                operand2: right_register,
                to: Local(format!("{}", result_register_id)),
                data_type: Type::Integer(Integer {
                    signed: true,
                    width: 32,
                }),
            })),
        "&" => ctx
            .current_basic_block
            .content
            .push(IRStatement::BinaryCalculate(BinaryCalculate {
                operation: BinaryOperation::And,
                operand1: left_register,
                operand2: right_register,
                to: Local(format!("{}", result_register_id)),
                data_type: Type::Integer(Integer {
                    signed: true,
                    width: 32,
                }),
            })),
        "^" => ctx
            .current_basic_block
            .content
            .push(IRStatement::BinaryCalculate(BinaryCalculate {
                operation: BinaryOperation::Xor,
                operand1: left_register,
                operand2: right_register,
                to: Local(format!("{}", result_register_id)),
                data_type: Type::Integer(Integer {
                    signed: true,
                    width: 32,
                }),
            })),
        "|" => ctx
            .current_basic_block
            .content
            .push(IRStatement::BinaryCalculate(BinaryCalculate {
                operation: BinaryOperation::Or,
                operand1: left_register,
                operand2: right_register,
                to: Local(format!("{}", result_register_id)),
                data_type: Type::Integer(Integer {
                    signed: true,
                    width: 32,
                }),
            })),
        _ => unimplemented!(),
    }
    LocalOrNumberLiteral::Local(Local(format!("{}", result_register_id)))
}
