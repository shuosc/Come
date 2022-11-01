use super::{register_assign, FunctionCompileContext};
use crate::ir;

pub mod basic_block;
pub mod statement;
pub mod terminator;

/// Emit assembly code for a [`ir::FunctionDefinition`].
pub fn emit_code(function: &ir::FunctionDefinition) -> String {
    let (register_assign, stack_space) = register_assign::assign_register(function);
    let mut result = format!("{}:\n", function.name);
    let mut context = FunctionCompileContext {
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
    }
    if stack_space != 0 {
        result.push_str(format!("    addi sp, sp, {}\n    ret\n", stack_space).as_str());
    }
    result
}
