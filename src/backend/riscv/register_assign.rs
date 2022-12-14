use core::fmt;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    iter, mem,
};

use indexmap::IndexSet;
use itertools::Itertools;
use petgraph::data;

use crate::{
    ir::{
        self,
        analyzer::{control_flow::ControlFlowGraph, register_usage::RegisterUsageAnalyzer},
        function::parameter::Parameter,
        statement::{IRStatement, IsIRStatement},
        RegisterName,
    },
    utility::data_type,
};

use super::{Context, HasSize};

/// How a logical register is mapped to real hardware register or memory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegisterAssign {
    /// The logical register is mapped to a hardware register.
    Register(String),
    /// The logical register is mapped to a set of hardware register.
    MultipleRegisters(Vec<String>),
    /// The logical register is actually alias to some stack space created by alloca and should only be used in `load` and `store`.
    StackRef(usize),
    /// The logical register is spilled to the stack.
    StackValue(usize),
}

impl fmt::Display for RegisterAssign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegisterAssign::Register(register) => write!(f, "{}", register),
            RegisterAssign::MultipleRegisters(registers) => write!(
                f,
                "{}",
                registers
                    .iter()
                    .map(|register| format!("{}", register))
                    .collect_vec()
                    .join(",")
            ),
            RegisterAssign::StackRef(offset) => write!(f, "alias to {}(sp)", offset),
            RegisterAssign::StackValue(offset) => write!(f, "{}(sp)", offset),
        }
    }
}

/// Assign registers for a [`ir::FunctionDefinition`].
pub fn assign_register(
    ir_code: &ir::FunctionDefinition,
    ctx: &Context,
    control_flow_graph: ControlFlowGraph,
    register_usage: RegisterUsageAnalyzer,
) -> (HashMap<ir::RegisterName, RegisterAssign>, usize) {
    let mut register_assign = assign_param(&ir_code.parameters, ctx);
    let mut current_used_stack_space = 0;
    let alloca_registers: Vec<_> = ir_code
        .iter()
        .filter_map(|it| it.try_as_alloca())
        .map(|it| it.generate_register().unwrap().0)
        .collect();
    let alloca_assign = assign_alloca(
        &alloca_registers,
        ctx,
        &register_usage,
        &mut current_used_stack_space,
    );
    register_assign.extend(alloca_assign);
    let consider_registers = register_usage
        .registers()
        .iter()
        .filter(|&&it| !alloca_registers.contains(it))
        .filter(|&&it| {
            ir_code
                .parameters
                .iter()
                .find(|param| it == &param.name)
                .is_none()
        })
        .cloned()
        .collect_vec();
    let variables_active_blocks: HashMap<_, HashSet<_>> = consider_registers
        .iter()
        .map(|&it| {
            (
                it.clone(),
                register_usage
                    .register_active_blocks(it, &control_flow_graph)
                    .into_iter()
                    .collect(),
            )
        })
        .collect();
    let mut register_groups = register_groups(
        &consider_registers,
        ir_code,
        ctx,
        &control_flow_graph,
        &register_usage,
    );
    register_groups.sort_by_cached_key(|group| {
        // todo: can be register usage count
        active_block_intersection(group, &variables_active_blocks).len()
    });
    let mut next_temporary_register_id = 2;
    for group in register_groups {
        let sample_register = group.iter().next().unwrap();

        let data_type = register_usage.get(sample_register).data_type();
        let type_bytes = (data_type.size(ctx) + 7) / 8;
        let need_registers = type_bytes / 4;

        let assigned_to_register = if next_temporary_register_id + need_registers - 1 <= 6 {
            let current_temporary_register_id = next_temporary_register_id;
            next_temporary_register_id += need_registers;
            if need_registers == 1 {
                RegisterAssign::Register(format!("t{}", current_temporary_register_id))
            } else {
                RegisterAssign::MultipleRegisters(
                    (current_temporary_register_id..current_temporary_register_id + need_registers)
                        .map(|it| format!("t{}", it))
                        .collect(),
                )
            }
        } else {
            let result = current_used_stack_space;
            current_used_stack_space += type_bytes;
            RegisterAssign::StackValue(result)
        };

        for register in group {
            register_assign.insert(register.clone(), assigned_to_register.clone());
        }
    }
    (register_assign, current_used_stack_space)
}

fn assign_param(params: &[Parameter], ctx: &Context) -> HashMap<ir::RegisterName, RegisterAssign> {
    let mut result = HashMap::new();
    let mut current_used_id = 0;
    for param in params {
        let type_bytes = (param.data_type.size(ctx) + 7) / 8;
        let need_registers = type_bytes / 4;
        let assigned_to_register = if need_registers == 1 {
            RegisterAssign::Register(format!("a{}", current_used_id))
        } else {
            RegisterAssign::MultipleRegisters(
                (current_used_id..current_used_id + need_registers)
                    .map(|it| format!("a{}", it))
                    .collect(),
            )
        };
        current_used_id += need_registers;
        result.insert(param.name.clone(), assigned_to_register);
    }
    result
}

fn assign_alloca(
    allocaed_registers: &[RegisterName],
    ctx: &Context,
    register_usage: &RegisterUsageAnalyzer,
    current_used_stack_space: &mut usize,
) -> HashMap<ir::RegisterName, RegisterAssign> {
    let mut result = HashMap::new();
    for register in allocaed_registers {
        let data_type = register_usage
            .register_usages()
            .get(register)
            .unwrap()
            .alloca_type();
        let type_bytes = (data_type.size(ctx) + 7) / 8;
        result.insert(
            register.clone(),
            RegisterAssign::StackRef(*current_used_stack_space),
        );
        *current_used_stack_space += type_bytes;
    }
    result
}

fn active_block_intersection(
    register_group: &HashSet<RegisterName>,
    registers_active_blocks: &HashMap<ir::RegisterName, HashSet<usize>>,
) -> HashSet<usize> {
    register_group
        .iter()
        .map(|it| registers_active_blocks.get(it).unwrap())
        .fold(HashSet::new(), |mut acc, x| {
            acc.extend(x);
            acc
        })
}

fn register_groups(
    consider_registers: &[&RegisterName],
    ir_code: &ir::FunctionDefinition,
    ctx: &Context,
    control_flow_graph: &ControlFlowGraph,
    register_usage: &RegisterUsageAnalyzer,
) -> Vec<HashSet<ir::RegisterName>> {
    let mut registers_active_block = HashMap::new();
    for &register in consider_registers {
        let active_blocks: HashSet<_> = register_usage
            .register_active_blocks(register, control_flow_graph)
            .into_iter()
            .collect();
        registers_active_block.insert(register.clone(), active_blocks);
    }
    // todo: collect_phied_registers result can also be mergered
    let mut register_groups = collect_phied_registers(ir_code);
    'a: for &register in consider_registers {
        for register_group in register_groups.iter() {
            if register_group.contains(register) {
                continue 'a;
            }
        }
        let data_type = register_usage.get(register).data_type();
        let type_bytes = (data_type.size(ctx) + 7) / 8;
        let need_registers = type_bytes / 4;

        if need_registers <= 1 {
            let register_active_block: HashSet<_> = register_usage
                .register_active_blocks(register, control_flow_graph)
                .into_iter()
                .collect();
            for register_group in register_groups.iter_mut() {
                let register_group_active_blocks =
                    active_block_intersection(register_group, &registers_active_block);
                if register_active_block
                    .intersection(&register_group_active_blocks)
                    .count()
                    == 0
                {
                    register_group.insert(register.clone());
                    continue 'a;
                }
            }
        }
        register_groups.push(iter::once(register.clone()).collect());
    }
    register_groups
}

fn collect_phied_registers(ir_code: &ir::FunctionDefinition) -> Vec<HashSet<ir::RegisterName>> {
    let mut phied_together_registers = Vec::new();
    for statement in ir_code.iter() {
        if let IRStatement::Phi(phi) = statement {
            let mut phied_regs: HashSet<_> = phi
                .from
                .iter()
                .filter_map(|it| it.name.clone().try_into().ok())
                .collect();
            phied_regs.insert(phi.to.clone());
            let mut existed = false;
            for phied_together_register_set in &mut phied_together_registers {
                if phied_regs.intersection(phied_together_register_set).count() != 0 {
                    phied_together_register_set.extend(mem::take(&mut phied_regs));
                    existed = true;
                    break;
                }
            }
            if !existed {
                phied_together_registers.push(phied_regs);
            }
        }
    }
    phied_together_registers
}

#[cfg(test)]
mod tests {
    use crate::{
        ir::{
            function::{basic_block::BasicBlock, test_util::*},
            statement::Ret,
            FunctionDefinition,
        },
        utility::data_type,
    };

    use super::{assign_register, *};

    #[test]
    fn test_collect_phied_registers() {
        let function_definition = FunctionDefinition {
            name: "f".to_string(),
            parameters: Vec::new(),
            return_type: data_type::Type::None,
            content: vec![
                BasicBlock {
                    name: Some("bb1".to_string()),
                    content: vec![
                        binop_constant("t0"),
                        binop_constant("a1"),
                        binop_constant("b0"),
                        jump("bb3"),
                    ],
                },
                BasicBlock {
                    name: Some("bb2".to_string()),
                    content: vec![
                        binop_constant("t1"),
                        binop_constant("a0"),
                        binop_constant("b1"),
                        jump("bb3"),
                    ],
                },
                BasicBlock {
                    name: Some("bb3".to_string()),
                    content: vec![
                        phi("t2", "bb1", "t0", "bb2", "t1"),
                        phi("a2", "bb1", "a1", "bb2", "a0"),
                        jump("bb5"),
                    ],
                },
                BasicBlock {
                    name: Some("bb4".to_string()),
                    content: vec![binop_constant("t3"), jump("bb5")],
                },
                BasicBlock {
                    name: Some("bb5".to_string()),
                    content: vec![
                        phi("t4", "bb3", "t2", "bb4", "t3"),
                        Ret { value: None }.into(),
                    ],
                },
            ],
        };
        let phied_together_registers = collect_phied_registers(&function_definition);
        assert_eq!(phied_together_registers.len(), 2);
        let contains_t0 = phied_together_registers
            .iter()
            .find(|it| it.contains(&RegisterName("t0".to_string())))
            .unwrap();
        assert_eq!(contains_t0.len(), 5);
        assert!(contains_t0.contains(&RegisterName("t1".to_string())));
        assert!(contains_t0.contains(&RegisterName("t2".to_string())));
        assert!(contains_t0.contains(&RegisterName("t3".to_string())));
        assert!(contains_t0.contains(&RegisterName("t4".to_string())));
        let contains_a0 = phied_together_registers
            .iter()
            .find(|it| it.contains(&RegisterName("a0".to_string())))
            .unwrap();
        assert!(contains_a0.contains(&RegisterName("a1".to_string())));
        assert!(contains_a0.contains(&RegisterName("a2".to_string())));
    }

    // #[test]
    // fn test_register_groups() {
    //     let function_definition = FunctionDefinition {
    //         name: "f".to_string(),
    //         parameters: Vec::new(),
    //         return_type: data_type::Type::None,
    //         content: vec![
    //             BasicBlock {
    //                 name: Some("bb1".to_string()),
    //                 content: vec![
    //                     binop_constant("t0"),
    //                     binop_constant("a1"),
    //                     binop_constant("b0"),
    //                     branch("bb2", "bb4"),
    //                 ],
    //             },
    //             BasicBlock {
    //                 name: Some("bb2".to_string()),
    //                 content: vec![
    //                     binop_constant("t1"),
    //                     binop_constant("a0"),
    //                     binop_constant("b1"),
    //                     jump("bb3"),
    //                 ],
    //             },
    //             BasicBlock {
    //                 name: Some("bb3".to_string()),
    //                 content: vec![
    //                     phi("t2", "bb1", "t0", "bb2", "t1"),
    //                     phi("a2", "bb1", "a1", "bb2", "a0"),
    //                     jump("bb5"),
    //                 ],
    //             },
    //             BasicBlock {
    //                 name: Some("bb4".to_string()),
    //                 content: vec![binop_constant("t3"), jump("bb5")],
    //             },
    //             BasicBlock {
    //                 name: Some("bb5".to_string()),
    //                 content: vec![
    //                     phi("t4", "bb3", "t2", "bb4", "t3"),
    //                     Ret { value: None }.into(),
    //                 ],
    //             },
    //         ],
    //     };
    //     let ctx = Context::default();
    //     let control_flow_graph = ControlFlowGraph::new(&function_definition);
    //     let register_analyzer = RegisterUsageAnalyzer::new(&function_definition);
    //     let groups = register_groups(
    //         &function_definition,
    //         &ctx,
    //         &control_flow_graph,
    //         &register_analyzer,
    //     );
    //     assert_eq!(groups.len(), 3);
    //     let contains_t0 = groups
    //         .iter()
    //         .find(|it| it.contains(&RegisterName("t0".to_string())))
    //         .unwrap();
    //     assert_eq!(contains_t0.len(), 5);
    //     assert!(contains_t0.contains(&RegisterName("t1".to_string())));
    //     assert!(contains_t0.contains(&RegisterName("t2".to_string())));
    //     assert!(contains_t0.contains(&RegisterName("t3".to_string())));
    //     assert!(contains_t0.contains(&RegisterName("t4".to_string())));

    //     let contains_a0 = groups
    //         .iter()
    //         .find(|it| it.contains(&RegisterName("a0".to_string())))
    //         .unwrap();
    //     assert!(contains_a0.contains(&RegisterName("a1".to_string())));
    //     assert!(contains_a0.contains(&RegisterName("a2".to_string())));

    //     let contains_b0 = groups
    //         .iter()
    //         .find(|it| it.contains(&RegisterName("b0".to_string())))
    //         .unwrap();
    //     assert_eq!(contains_b0.len(), 2);
    //     assert!(contains_b0.contains(&RegisterName("b1".to_string())));

    //     let function_definition = FunctionDefinition {
    //         name: "f".to_string(),
    //         parameters: Vec::new(),
    //         return_type: data_type::Type::None,
    //         content: vec![
    //             BasicBlock {
    //                 name: Some("bb0".to_string()),
    //                 content: vec![
    //                     binop_constant("m"),
    //                     binop_constant("n"),
    //                     binop_constant("u1"),
    //                     binop("i0", "m", "m"),
    //                     binop("j0", "n", "n"),
    //                     binop("a0", "u1", "u1"),
    //                     binop_constant("r"),
    //                     jump("bb1"),
    //                 ],
    //             },
    //             BasicBlock {
    //                 name: Some("bb1".to_string()),
    //                 content: vec![
    //                     phi("i_bb1", "bb1", "i0", "bb4", "i2"),
    //                     phi("a_bb1", "bb1", "a0", "bb4", "a1"),
    //                     binop("i1", "i_bb1", "i_bb1"),
    //                     binop("j1", "j0", "j0"),
    //                     branch("bb2", "bb3"),
    //                 ],
    //             },
    //             BasicBlock {
    //                 name: Some("bb2".to_string()),
    //                 content: vec![
    //                     binop("u2", "a_bb1", "a_bb1"),
    //                     binop("a1", "u2", "i1"),
    //                     jump("bb3"),
    //                 ],
    //             },
    //             BasicBlock {
    //                 name: Some("bb3".to_string()),
    //                 content: vec![
    //                     binop_constant("u3"),
    //                     binop("i2", "u3", "j1"),
    //                     branch("bb1", "bb4"),
    //                 ],
    //             },
    //             BasicBlock {
    //                 name: Some("bb4".to_string()),
    //                 content: vec![Ret {
    //                     value: Some(RegisterName("r".to_string()).into()),
    //                 }
    //                 .into()],
    //             },
    //         ],
    //     };
    //     let ctx = Context::default();
    //     let control_flow_graph = ControlFlowGraph::new(&function_definition);
    //     let register_analyzer = RegisterUsageAnalyzer::new(&function_definition);
    //     let groups = register_groups(
    //         &function_definition,
    //         &ctx,
    //         &control_flow_graph,
    //         &register_analyzer,
    //     );
    //     assert_eq!(groups.len(), 7);
    // }

    // #[test]
    // fn test_assign_register() {
    //     let function_definition = FunctionDefinition {
    //         name: "f".to_string(),
    //         parameters: Vec::new(),
    //         return_type: data_type::Type::None,
    //         content: vec![
    //             BasicBlock {
    //                 name: Some("bb0".to_string()),
    //                 content: vec![
    //                     binop_constant("m"),
    //                     binop_constant("n"),
    //                     binop_constant("u1"),
    //                     binop("i0", "m", "m"),
    //                     binop("j0", "n", "n"),
    //                     binop("a0", "u1", "u1"),
    //                     binop_constant("r"),
    //                     jump("bb1"),
    //                 ],
    //             },
    //             BasicBlock {
    //                 name: Some("bb1".to_string()),
    //                 content: vec![
    //                     phi("i_bb1", "bb1", "i0", "bb4", "i2"),
    //                     phi("a_bb1", "bb1", "a0", "bb4", "a1"),
    //                     binop("i1", "i_bb1", "i_bb1"),
    //                     binop("j1", "j0", "j0"),
    //                     branch("bb2", "bb3"),
    //                 ],
    //             },
    //             BasicBlock {
    //                 name: Some("bb2".to_string()),
    //                 content: vec![
    //                     binop("u2", "a_bb1", "a_bb1"),
    //                     binop("a1", "u2", "i1"),
    //                     jump("bb3"),
    //                 ],
    //             },
    //             BasicBlock {
    //                 name: Some("bb3".to_string()),
    //                 content: vec![
    //                     binop_constant("u3"),
    //                     binop("i2", "u3", "j1"),
    //                     branch("bb1", "bb4"),
    //                 ],
    //             },
    //             BasicBlock {
    //                 name: Some("bb4".to_string()),
    //                 content: vec![Ret {
    //                     value: Some(RegisterName("r".to_string()).into()),
    //                 }
    //                 .into()],
    //             },
    //         ],
    //     };
    //     let ctx = Context::default();
    //     let control_flow_graph = ControlFlowGraph::new(&function_definition);
    //     let register_analyzer = RegisterUsageAnalyzer::new(&function_definition);
    //     let assign = assign_register(
    //         &function_definition,
    //         &ctx,
    //         control_flow_graph,
    //         register_analyzer,
    //     );
    // }

}
