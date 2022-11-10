use super::IRGeneratingContext;
use crate::{
    ast::{self, expression::RValue},
    ir::{
        function::statement::{calculate, Load},
        quantity::Quantity,
        statement::load_field,
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
        ast::expression::rvalue::RValue::FunctionCall(_function_call) => {
            todo!()
        }
        ast::expression::rvalue::RValue::InBrackets(x) => rvalue_from_ast(&x.0, ctx),
        ast::expression::rvalue::RValue::FieldAccess(field_access) => {
            load_field::from_ast(field_access, ctx).into()
        }
        ast::expression::rvalue::RValue::UnaryOperatorResult(unary_operator_result) => {
            calculate::unary::from_ast(unary_operator_result, ctx)
        }
        ast::expression::rvalue::RValue::BinaryOperatorResult(binary_operator_result) => {
            calculate::binary::from_ast(binary_operator_result, ctx).into()
        }
    }
}
// todo: tests
