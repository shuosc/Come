use super::IRGeneratingContext;
use crate::{
    ast::{
        self,
        expression::{LValue, RValue},
    },
    ir::{
        function::statement::{calculate, Load},
        quantity::Quantity,
        statement::load_field,
        LocalVariableName,
    },
    utility::data_type::Type,
};

/// Generate IR from an [`ast::expression::RValue`] AST node.
/// Return the register where the result is stored.
pub fn rvalue_from_ast(ast: &RValue, ctx: &mut IRGeneratingContext) -> Quantity {
    match ast {
        RValue::IntegerLiteral(number_literal) => number_literal.0.into(),
        RValue::VariableRef(variable_ref) => {
            let data_type = ctx.type_of_variable(variable_ref);
            let target = ctx.next_register_with_type(&data_type);
            let source = LocalVariableName(format!("{}_addr", variable_ref.0.clone()));
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

/// Generate IR from an [`ast::expression::LValue`] AST node.
/// Return the register where the result address is stored.
pub fn lvalue_from_ast(
    ast: &ast::expression::lvalue::LValue,
    ctx: &mut IRGeneratingContext,
) -> Quantity {
    match ast {
        ast::expression::LValue::VariableRef(variable_ref) => {
            let result = LocalVariableName(format!("{}_addr", variable_ref.0));
            ctx.local_variable_types
                .insert(result.clone(), Type::Address);
            result.into()
        }
        ast::expression::LValue::FieldAccess(field_access) => {
            // for field access, we will return the "root" object's address
            let mut root = field_access.from.as_ref();
            while let LValue::FieldAccess(field_access) = &root {
                root = field_access.from.as_ref();
            }
            lvalue_from_ast(root, ctx)
        }
    }
}

// todo: tests
