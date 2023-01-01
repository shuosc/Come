use super::IRGeneratingContext;
use crate::{
    ast::expression::RValue,
    ir::{
        function::statement::{calculate, Load},
        quantity::Quantity,
        statement::{call, load_field},
    },
};

/// Generate IR from an [`ast::expression::RValue`] AST node.
/// Return the register where the result is stored.
pub fn rvalue_from_ast(ast: &RValue, ctx: &mut IRGeneratingContext) -> Quantity {
    match ast {
        RValue::IntegerLiteral(number_literal) => number_literal.0.into(),
        RValue::VariableRef(variable_ref) => {
            let data_type = ctx.symbol_table.type_of_variable(variable_ref);
            let target = ctx.next_register_with_type(&data_type);
            let source = ctx
                .symbol_table
                .current_variable_address_register(variable_ref);
            ctx.current_basic_block.append_statement(Load {
                from: source.into(),
                to: target.clone(),
                data_type,
            });
            target.into()
        }
        RValue::FunctionCall(function_call) => call::from_ast(function_call, ctx).into(),
        RValue::InBrackets(x) => rvalue_from_ast(&x.0, ctx),
        RValue::FieldAccess(field_access) => load_field::from_ast(field_access, ctx).into(),
        RValue::UnaryOperatorResult(unary_operator_result) => {
            calculate::unary::from_ast(unary_operator_result, ctx)
        }
        RValue::BinaryOperatorResult(binary_operator_result) => {
            calculate::binary::from_ast(binary_operator_result, ctx).into()
        }
    }
}
// todo: tests
