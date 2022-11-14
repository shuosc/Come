use crate::ir::statement::Ret;

use super::*;

/// Generate IR from an [`ast::statement::Return`] AST node.
pub fn from_ast(ast: &ast::statement::Return, ctx: &mut IRGeneratingContext) {
    let ast::statement::Return(value) = ast;
    let return_value = value
        .clone()
        .map(|value| expression::rvalue_from_ast(&value, ctx));
    ctx.end_current_basic_block_with(Ret {
        value: return_value,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ast::expression::IntegerLiteral, ir::statement};

    #[test]
    fn test_from_ast() {
        let mut parent_ctx = crate::ir::IRGeneratingContext::new();
        let mut ctx = IRGeneratingContext::new(&mut parent_ctx);
        let ast = ast::statement::return_statement::Return(Some(IntegerLiteral(42).into()));
        from_ast(&ast, &mut ctx);
        let basic_blocks = ctx.done();
        assert_eq!(
            basic_blocks[0].content.last().unwrap().clone(),
            statement::Ret {
                value: Some(42.into())
            }
            .into()
        );
    }
}
