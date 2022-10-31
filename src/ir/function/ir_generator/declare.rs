use super::IRGeneratingContext;
use crate::{
    ast,
    ir::{function::statement::Alloca, quantity::local::LocalVariableName},
};

/// Generate IR from an [`ast::statement::Declare`] AST node.
pub fn from_ast(ast: &ast::statement::declare::Declare, ctx: &mut IRGeneratingContext) {
    let ast::statement::Declare {
        variable_name,
        data_type,
        init_value,
    } = ast;
    ctx.current_basic_block.append_statement(Alloca {
        to: LocalVariableName(format!("{}_addr", variable_name)),
        alloc_type: data_type.clone(),
    });
    if let Some(init_value) = init_value {
        // create a dummy assign node
        let assign_statement = ast::statement::Assign {
            lhs: ast::expression::lvalue::LValue::VariableRef(
                ast::expression::variable_ref::VariableRef(variable_name.clone()),
            ),
            rhs: init_value.clone(),
        };
        // and generate its ast
        super::assign::from_ast(&assign_statement, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ast::expression::IntegerLiteral,
        ir::statement,
        utility::data_type::{Integer, Type},
    };

    #[test]
    fn test_from_ast() {
        let mut parent_ctx = crate::ir::IRGeneratingContext::new();
        let mut ctx = IRGeneratingContext::new(&mut parent_ctx);
        let ast = ast::statement::Declare {
            variable_name: "a".to_string(),
            data_type: Type::Integer(Integer {
                signed: true,
                width: 32,
            }),
            init_value: Some(IntegerLiteral(42).into()),
        };
        from_ast(&ast, &mut ctx);
        assert_eq!(ctx.current_basic_block.content.len(), 2);
        assert_eq!(
            ctx.current_basic_block.content[0],
            Alloca {
                to: LocalVariableName("a_addr".to_string()),
                alloc_type: Type::Integer(Integer {
                    signed: true,
                    width: 32,
                }),
            }
            .into()
        );
        assert_eq!(
            ctx.current_basic_block.content[1],
            statement::Store {
                source: 42.into(),
                target: LocalVariableName("a_addr".to_string()).into(),
                data_type: Type::Integer(Integer {
                    signed: true,
                    width: 32,
                }),
            }
            .into()
        );
    }
}
