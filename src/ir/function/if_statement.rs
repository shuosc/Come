use std::mem;

use crate::ir::statements::Branch;

use super::*;

pub fn from_ast(ast: &ast::statement::if_statement::If, ctx: &mut IRGeneratingContext) {
    let ast::statement::if_statement::If {
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
    ctx.current_basic_block.terminator = Some(crate::ir::statements::Terminator::Branch(Branch {
        branch_type: crate::ir::statements::branch::BranchType::NE,
        operand1: condition,
        operand2: LocalOrNumberLiteral::NumberLiteral(0),
        success_label: success_label.clone(),
        failure_label: fail_label.clone(),
    }));
    ctx.done_basic_blocks.push(mem::replace(
        &mut ctx.current_basic_block,
        BasicBlock {
            name: Some(success_label),
            phis: Vec::new(),
            content: Vec::new(),
            terminator: None,
        },
    ));
    compound_from_ast(content, ctx);
    ctx.current_basic_block.terminator = Some(crate::ir::statements::Terminator::Jump(
        crate::ir::statements::Jump {
            label: end_label.clone(),
        },
    ));
    ctx.done_basic_blocks.push(mem::replace(
        &mut ctx.current_basic_block,
        BasicBlock {
            name: None,
            phis: Vec::new(),
            content: Vec::new(),
            terminator: None,
        },
    ));
    if let Some(else_content) = else_content {
        ctx.current_basic_block.name = Some(fail_label);
        compound_from_ast(else_content, ctx);
        ctx.current_basic_block.terminator = Some(crate::ir::statements::Terminator::Jump(
            crate::ir::statements::Jump {
                label: end_label.clone(),
            },
        ));
        ctx.done_basic_blocks.push(mem::replace(
            &mut ctx.current_basic_block,
            BasicBlock {
                name: None,
                phis: Vec::new(),
                content: Vec::new(),
                terminator: None,
            },
        ));
    }
    ctx.current_basic_block.name = Some(end_label);
}
