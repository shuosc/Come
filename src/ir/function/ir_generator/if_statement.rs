use super::{compound_from_ast, expression, IRGeneratingContext};
use crate::{
    ast,
    ir::statement::{branch::BranchType, Branch, Jump},
};

/// Generates IR for an if statement.
pub fn from_ast(ast: &ast::statement::If, ctx: &mut IRGeneratingContext) {
    let ast::statement::If {
        condition,
        content,
        else_content,
    } = ast;

    let if_id = ctx.parent_context.next_if_id;
    ctx.parent_context.next_if_id += 1;
    let success_label = format!("if_{}_success", if_id);
    let fail_label = format!("if_{}_fail", if_id);
    let end_label = format!("if_{}_end", if_id);
    let condition = expression::rvalue_from_ast(condition, ctx);
    ctx.end_current_basic_block_with(Branch {
        branch_type: BranchType::NE,
        operand1: condition,
        operand2: 0.into(),
        success_label: success_label.clone(),
        failure_label: fail_label.clone(),
    });
    ctx.current_basic_block.name = Some(success_label);
    compound_from_ast(content, ctx);
    // it is possible that last BasicBlock has already end
    if !ctx.current_basic_block.empty() {
        ctx.end_current_basic_block_with(Jump {
            label: end_label.clone(),
        });
    }
    ctx.current_basic_block.name = Some(fail_label);
    if let Some(else_content) = else_content {
        compound_from_ast(else_content, ctx);
    }
    if !ctx.current_basic_block.empty() {
        ctx.end_current_basic_block_with(Jump {
            label: end_label.clone(),
        });
    }
    ctx.current_basic_block.name = Some(end_label);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ast::expression::{IntegerLiteral, VariableRef},
        ir::statement,
        utility::data_type,
    };

    #[test]
    fn test_from_ast() {
        let mut parent_ctx = crate::ir::IRGeneratingContext::new();
        let mut ctx = IRGeneratingContext::new(&mut parent_ctx);
        ctx.symbol_table
            .variable_types_stack
            .last_mut()
            .unwrap()
            .insert(VariableRef("a".to_string()), (data_type::I32.clone(), 0));
        ctx.symbol_table
            .variable_types_stack
            .last_mut()
            .unwrap()
            .insert(VariableRef("b".to_string()), (data_type::I32.clone(), 0));
        let ast = ast::statement::If {
            condition: IntegerLiteral(42).into(),
            content: ast::statement::compound::Compound(vec![ast::statement::Assign {
                lhs: ast::expression::lvalue::LValue::VariableRef(
                    ast::expression::variable_ref::VariableRef("a".to_string()),
                ),
                rhs: IntegerLiteral(42).into(),
            }
            .into()]),
            else_content: Some(ast::statement::compound::Compound(vec![
                ast::statement::Assign {
                    lhs: ast::expression::lvalue::LValue::VariableRef(
                        ast::expression::variable_ref::VariableRef("b".to_string()),
                    ),
                    rhs: IntegerLiteral(42).into(),
                }
                .into(),
            ])),
        };
        from_ast(&ast, &mut ctx);
        let basic_blocks = ctx.done();
        assert_eq!(basic_blocks.len(), 4);
        assert_eq!(basic_blocks[0].content.len(), 1);
        assert_eq!(
            basic_blocks[0].content[0].clone(),
            statement::Branch {
                branch_type: crate::ir::statement::branch::BranchType::NE,
                operand1: 42.into(),
                operand2: 0.into(),
                success_label: "if_0_success".to_string(),
                failure_label: "if_0_fail".to_string(),
            }
            .into()
        );
        assert_eq!(basic_blocks[1].name.as_ref().unwrap(), "if_0_success");
        assert_eq!(basic_blocks[1].content.len(), 2);
        assert_eq!(
            basic_blocks[1].content[1].clone(),
            statement::Jump {
                label: "if_0_end".to_string(),
            }
            .into()
        );
        assert_eq!(basic_blocks[2].name.as_ref().unwrap(), "if_0_fail");
        assert_eq!(basic_blocks[2].content.len(), 2);
        assert_eq!(
            basic_blocks[2].content[1].clone(),
            statement::Jump {
                label: "if_0_end".to_string(),
            }
            .into()
        );
        assert_eq!(basic_blocks[3].name.as_ref().unwrap(), "if_0_end");
    }
}
