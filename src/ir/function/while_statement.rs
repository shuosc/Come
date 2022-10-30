use std::mem;

use crate::ir::statements::{branch::BranchType, Branch, Jump, Terminator};

use super::{expression::rvalue_from_ast, *};
pub fn from_ast(ast: &ast::statement::while_statement::While, ctx: &mut IRGeneratingContext) {
    let ast::statement::while_statement::While {
        condition,
        content: _,
    } = ast;
    let statement_id = ctx.parent_context.next_loop_id;
    ctx.parent_context.next_loop_id += 1;
    let condition_label = format!("loop_{}_condition", statement_id);
    let success_label = format!("loop_{}_success", statement_id);
    let fail_label = format!("loop_{}_fail", statement_id);
    ctx.current_basic_block.terminator = Some(Terminator::Jump(Jump {
        label: condition_label.clone(),
    }));
    ctx.done_basic_blocks.push(mem::replace(
        &mut ctx.current_basic_block,
        BasicBlock {
            name: Some(condition_label.clone()),
            phis: Vec::new(),
            content: Vec::new(),
            terminator: None,
        },
    ));
    let condition_register = rvalue_from_ast(condition, ctx);
    ctx.current_basic_block.terminator = Some(Terminator::Branch(Branch {
        branch_type: BranchType::NE,
        operand1: condition_register,
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
    ctx.current_basic_block.terminator = Some(Terminator::Jump(Jump {
        label: condition_label,
    }));
    ctx.done_basic_blocks.push(mem::replace(
        &mut ctx.current_basic_block,
        BasicBlock {
            name: Some(fail_label),
            phis: Vec::new(),
            content: Vec::new(),
            terminator: None,
        },
    ));
}
