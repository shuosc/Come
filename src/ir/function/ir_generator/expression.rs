use super::IRGeneratingContext;
use crate::{
    ast::{self, expression::RValue},
    ir::{
        function::statement::{calculate, Load},
        quantity::Quantity,
        LocalVariableName,
    },
    utility::data_type::{Integer, Type},
};

/// Generate IR from an [`ast::expression::RValue`] AST node.
/// Return the register where the result is stored.
pub fn rvalue_from_ast(ast: &RValue, ctx: &mut IRGeneratingContext) -> Quantity {
    match ast {
        RValue::IntegerLiteral(number_literal) => number_literal.0.into(),
        RValue::VariableRef(variable_ref) => {
            let target = ctx.parent_context.next_register();
            let source = LocalVariableName(format!("{}_addr", variable_ref.0.clone()));
            ctx.current_basic_block.append_statement(Load {
                from: source.into(),
                to: target.clone(),
                data_type: Type::Integer(Integer {
                    signed: true,
                    width: 32,
                }),
            });
            target.into()
        }
        ast::expression::rvalue::RValue::FunctionCall(_function_call) => {
            todo!()
        }
        ast::expression::rvalue::RValue::InBrackets(x) => rvalue_from_ast(&x.0, ctx),
        ast::expression::rvalue::RValue::FieldAccess(_) => todo!(),
        ast::expression::rvalue::RValue::UnaryOperatorResult(unary_operator_result) => {
            calculate::unary::from_ast(unary_operator_result, ctx)
        }
        ast::expression::rvalue::RValue::BinaryOperatorResult(binary_operator_result) => {
            calculate::binary::from_ast(binary_operator_result, ctx).into()
        }
    }
}

/// Generate IR from an [`ast::expression::LValue`] AST node.
/// Return the register where the result address is stored.
pub fn lvalue_from_ast(
    ast: &ast::expression::lvalue::LValue,
    _ctx: &mut IRGeneratingContext,
) -> Quantity {
    if let ast::expression::LValue::VariableRef(variable_ref) = ast {
        LocalVariableName(format!("{}_addr", variable_ref.0)).into()
    } else {
        unimplemented!()
    }
}

// todo: tests
