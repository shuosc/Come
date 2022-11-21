use super::{compound_from_ast, expression::rvalue_from_ast, IRGeneratingContext};
use crate::{
    ast,
    ir::statement::{branch::BranchType, Branch, Jump},
};

/// Generate IR from [`ast::statement::While`] AST node.
pub fn from_ast(ast: &ast::statement::While, ctx: &mut IRGeneratingContext) {
    let ast::statement::While { condition, content } = ast;
    let statement_id = ctx.parent_context.next_loop_id;
    ctx.parent_context.next_loop_id += 1;
    let condition_label = format!("loop_{}_condition", statement_id);
    let success_label = format!("loop_{}_success", statement_id);
    let fail_label = format!("loop_{}_fail", statement_id);
    ctx.end_current_basic_block_with(Jump {
        label: condition_label.clone(),
    });
    ctx.current_basic_block.name = Some(condition_label.clone());
    let condition_register = rvalue_from_ast(condition, ctx);
    ctx.end_current_basic_block_with(Branch {
        branch_type: BranchType::NE,
        operand1: condition_register,
        operand2: 0.into(),
        success_label: success_label.clone(),
        failure_label: fail_label.clone(),
    });
    ctx.current_basic_block.name = Some(success_label);
    compound_from_ast(content, ctx);
    if !ctx.current_basic_block.empty() {
        ctx.end_current_basic_block_with(Jump {
            label: condition_label,
        });
    }
    ctx.current_basic_block.name = Some(fail_label);
}

#[cfg(test)]
mod tests {
    #![allow(clippy::borrow_interior_mutable_const)]
    use super::*;
    use crate::{
        ast::expression::{IntegerLiteral, VariableRef},
        ir::statement::Ret,
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
        let ast = ast::statement::While {
            condition: IntegerLiteral(42).into(),
            content: ast::statement::compound::Compound(vec![ast::statement::Assign {
                lhs: ast::expression::lvalue::LValue::VariableRef(
                    ast::expression::variable_ref::VariableRef("a".to_string()),
                ),
                rhs: IntegerLiteral(42).into(),
            }
            .into()]),
        };
        from_ast(&ast, &mut ctx);
        let basic_blocks = ctx.done();
        assert_eq!(basic_blocks.len(), 4);
        assert_eq!(basic_blocks[1].name.as_ref().unwrap(), "loop_0_condition");
        assert_eq!(
            basic_blocks[1].content.last().unwrap().clone(),
            Branch {
                branch_type: BranchType::NE,
                operand1: 42.into(),
                operand2: 0.into(),
                success_label: "loop_0_success".to_string(),
                failure_label: "loop_0_fail".to_string(),
            }
            .into()
        );
        assert_eq!(basic_blocks[2].name.as_ref().unwrap(), "loop_0_success");
        assert_eq!(basic_blocks[2].content.len(), 2);
        assert_eq!(
            basic_blocks[2].content.last().unwrap().clone(),
            Jump {
                label: "loop_0_condition".to_string(),
            }
            .into()
        );
        assert_eq!(basic_blocks[3].name.as_ref().unwrap(), "loop_0_fail");
        assert_eq!(
            basic_blocks[3].content.last().unwrap().clone(),
            Ret { value: None }.into()
        );
    }
}
