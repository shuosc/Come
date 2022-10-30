use crate::{
    ast,
    ir::{
        quantity::{local::Local, LocalOrGlobal, LocalOrNumberLiteral},
        statements::{IRStatement, Load},
    },
    utility::data_type::{Integer, Type},
};

use super::IRGeneratingContext;

mod binary_operator_result;
mod unary_operator_result;

pub fn rvalue_from_ast(
    ast: &ast::expression::rvalue::RValue,
    ctx: &mut IRGeneratingContext,
) -> LocalOrNumberLiteral {
    match ast {
        ast::expression::rvalue::RValue::IntegerLiteral(number_literal) => {
            LocalOrNumberLiteral::NumberLiteral(number_literal.0)
        }
        ast::expression::rvalue::RValue::VariableRef(variable_ref) => {
            let next_register_id = ctx.parent_context.next_register_id;
            ctx.parent_context.next_register_id += 1;
            let source = Local(format!("{}_addr", variable_ref.0.clone()));
            let target = Local(format!("{}", next_register_id));
            ctx.current_basic_block
                .content
                .push(IRStatement::Load(Load {
                    from: LocalOrGlobal::Local(source),
                    to: target.clone(),
                    data_type: Type::Integer(Integer {
                        signed: true,
                        width: 32,
                    }),
                }));
            LocalOrNumberLiteral::Local(target)
        }
        ast::expression::rvalue::RValue::FunctionCall(_function_call) => {
            todo!()
        }
        ast::expression::rvalue::RValue::InBrackets(x) => rvalue_from_ast(&x.0, ctx),
        ast::expression::rvalue::RValue::FieldAccess(_) => todo!(),
        ast::expression::rvalue::RValue::UnaryOperatorResult(unary_operator_result) => {
            unary_operator_result::from_ast(unary_operator_result, ctx)
        }
        ast::expression::rvalue::RValue::BinaryOperatorResult(binary_operator_result) => {
            binary_operator_result::from_ast(binary_operator_result, ctx)
        }
    }
}

pub fn lvalue_from_ast(
    ast: &ast::expression::lvalue::LValue,
    _ctx: &mut IRGeneratingContext,
) -> LocalOrGlobal {
    if let ast::expression::lvalue::LValue::VariableRef(variable_ref) = ast {
        LocalOrGlobal::Local(Local(format!("{}_addr", variable_ref.0)))
    } else {
        unimplemented!()
    }
}
