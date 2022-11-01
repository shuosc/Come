use super::{lvalue_from_ast, rvalue_from_ast, IRGeneratingContext};
use crate::{
    ast,
    ir::function::statement,
    utility::data_type::{Integer, Type},
};

/// Generate IR from an [`ast::statement::Assign`] AST node.
pub fn from_ast(ast: &ast::statement::Assign, ctx: &mut IRGeneratingContext) {
    // generate rhs code and get the result register
    let result_register = rvalue_from_ast(&ast.rhs, ctx);
    // generate lhs code and get the result register
    let lhs_address_register = lvalue_from_ast(&ast.lhs, ctx);
    // generate store code
    ctx.current_basic_block.append_statement(statement::Store {
        source: result_register,
        target: lhs_address_register,
        // todo: use real datatype
        data_type: Type::Integer(Integer {
            signed: true,
            width: 32,
        }),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ast::expression::{IntegerLiteral, VariableRef},
        ir::LocalVariableName,
    };

    #[test]
    fn test_from_ast() {
        let mut parent_ctx = crate::ir::IRGeneratingContext::new();
        let mut ctx = IRGeneratingContext::new(&mut parent_ctx);
        let ast = ast::statement::Assign {
            lhs: VariableRef("a".to_string()).into(),
            rhs: IntegerLiteral(42).into(),
        };
        from_ast(&ast, &mut ctx);
        let basic_blocks = ctx.done();
        assert_eq!(basic_blocks.len(), 1);
        assert_eq!(
            basic_blocks[0].content[0],
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
