use std::collections::HashMap;

use super::register_assign::{self, RegisterAssign};
use crate::ir::{
    self,
    analyzer::{control_flow::ControlFlowGraph, register_usage::RegisterUsageAnalyzer},
    quantity::Quantity,
    statement::{IRStatement, Phi},
    RegisterName,
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
    pub phi_constant_assign: HashMap<String, Vec<(RegisterAssign, i64)>>,
}

fn collect_phi_constant_assign(
    function: &ir::FunctionDefinition,
    register_assign: &HashMap<RegisterName, RegisterAssign>,
) -> HashMap<String, Vec<(RegisterAssign, i64)>> {
    let mut result: HashMap<String, Vec<(RegisterAssign, i64)>> = HashMap::new();
    for statement in function.iter() {
        if let IRStatement::Phi(Phi { to, from, .. }) = statement {
            for from in from {
                if let Quantity::NumberLiteral(n) = from.value {
                    result
                        .entry(from.block.clone())
                        .or_default()
                        .push((register_assign[to].clone(), n));
                }
            }
        }
    }
    result
}

/// Emit assembly code for a [`ir::FunctionDefinition`].
pub fn emit_code(function: &ir::FunctionDefinition, ctx: &mut super::Context) -> String {
    let control_flow_graph = ControlFlowGraph::new(function);
    let register_usage = RegisterUsageAnalyzer::new(function);
    let (register_assign, stack_space) =
        register_assign::assign_register(ctx, function, control_flow_graph, register_usage);
    let phi_constant_assign = collect_phi_constant_assign(function, &register_assign);
    let mut result = format!(
        ".global {}\n{}:\n",
        function.header.name, function.header.name
    );
    let mut context = FunctionCompileContext {
        parent_context: ctx,
        local_assign: register_assign,
        cleanup_label: if stack_space != 0 {
            Some(format!("{}_end", function.header.name))
        } else {
            None
        },
        phi_constant_assign,
    };
    if stack_space != 0 {
        result.push_str(format!("    addi sp, sp, -{stack_space}\n").as_str());
    }
    for basic_block in function.content.iter() {
        result.push_str(basic_block::emit_code(basic_block, &mut context).as_str());
    }
    if let Some(cleanup_label) = context.cleanup_label {
        result.push_str(format!("{cleanup_label}:\n").as_str());
        if stack_space != 0 {
            result.push_str(format!("    addi sp, sp, {stack_space}\n").as_str());
        }
        result.push_str("    ret\n");
    }
    result
}

#[cfg(test)]
mod tests {
    #![allow(clippy::borrow_interior_mutable_const)]

    use crate::{
        ir::{
            function::{basic_block::BasicBlock, test_util::*},
            statement::phi::PhiSource,
        },
        utility::data_type::{self, Type},
    };

    use super::*;

    #[test]
    fn test_collect_phi_constant_assign() {
        let function = ir::FunctionDefinition {
            header: ir::FunctionHeader {
                name: "f".to_string(),
                parameters: Vec::new(),
                return_type: Type::None,
            },
            content: vec![
                BasicBlock {
                    name: Some("f_entry".to_string()),
                    content: vec![branch("bb1", "bb2")],
                },
                BasicBlock {
                    name: Some("bb1".to_string()),
                    content: vec![jump("bb3")],
                },
                BasicBlock {
                    name: Some("bb2".to_string()),
                    content: vec![jump("bb3")],
                },
                BasicBlock {
                    name: Some("bb3".to_string()),
                    content: vec![Phi {
                        to: RegisterName("reg0".to_string()),
                        data_type: data_type::I32.clone(),
                        from: vec![
                            PhiSource {
                                value: 1.into(),
                                block: "bb1".to_string(),
                            },
                            PhiSource {
                                value: 2.into(),
                                block: "bb2".to_string(),
                            },
                        ],
                    }
                    .into()],
                },
            ],
        };
        let mut register_assign = HashMap::new();
        register_assign.insert(
            RegisterName("reg0".to_string()),
            RegisterAssign::Register("t0".to_string()),
        );
        let result = collect_phi_constant_assign(&function, &register_assign);
        let bb1_result = result.get("bb1").unwrap();
        assert_eq!(bb1_result.len(), 1);
        assert_eq!(bb1_result[0].0, RegisterAssign::Register("t0".to_string()));
        assert_eq!(bb1_result[0].1, 1);
        let bb2_result = result.get("bb2").unwrap();
        assert_eq!(bb2_result.len(), 1);
        assert_eq!(bb2_result[0].0, RegisterAssign::Register("t0".to_string()));
        assert_eq!(bb2_result[0].1, 2);
    }
}
