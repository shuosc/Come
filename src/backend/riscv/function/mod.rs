use std::collections::HashMap;

use super::register_assign::{self, RegisterAssign};
use crate::ir::{
    self,
    analyzer::{control_flow::ControlFlowGraph, register_usage::RegisterUsageAnalyzer},
};

pub mod basic_block;
pub mod statement;

/// Context for compiling a function.
pub struct FunctionCompileContext<'a> {
    /// Parent context
    pub parent_context: &'a mut super::Context,
    /// Where a local variable is assigned to.
    pub local_assign: HashMap<ir::RegisterName, RegisterAssign>,
    /// Some times we need to do some cleanup before return (eg, pop the stack frame)
    /// So we can jump to this label instead of return directly.
    pub cleanup_label: Option<String>,
}

/// Emit assembly code for a [`ir::FunctionDefinition`].
pub fn emit_code(function: &ir::FunctionDefinition, ctx: &mut super::Context) -> String {
    let control_flow_graph = ControlFlowGraph::new(function);
    let register_usage = RegisterUsageAnalyzer::new(function);
    let (register_assign, stack_space) =
        register_assign::assign_register(ctx, function, control_flow_graph, register_usage);
    let mut result = format!("{}:\n", function.name);
    let mut context = FunctionCompileContext {
        parent_context: ctx,
        local_assign: register_assign,
        cleanup_label: if stack_space != 0 {
            Some(format!("{}_end", function.name))
        } else {
            None
        },
    };
    if stack_space != 0 {
        result.push_str(format!("    addi sp, sp, -{}\n", stack_space).as_str());
    }
    for basic_block in function.content.iter() {
        result.push_str(basic_block::emit_code(basic_block, &mut context).as_str());
    }
    if let Some(cleanup_label) = context.cleanup_label {
        result.push_str(format!("{}:\n", cleanup_label).as_str());
        if stack_space != 0 {
            result.push_str(format!("    addi sp, sp, {}\n", stack_space).as_str());
        }
        result.push_str("    ret\n");
    }
    result
}
