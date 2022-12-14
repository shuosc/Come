use std::{
    collections::{HashMap, HashSet},
    iter, mem,
};

use itertools::Itertools;

use crate::ir::{
    self,
    analyzer::{control_flow::ControlFlowGraph, register_usage::RegisterUsageAnalyzer},
    statement::{IRStatement, IsIRStatement},
    RegisterName,
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

/// Assign registers for a [`ir::FunctionDefinition`].
pub fn assign_register(
    ir_code: &ir::FunctionDefinition,
    ctx: &Context,
    control_flow_graph: ControlFlowGraph,
    register_usage: RegisterUsageAnalyzer,
) -> (HashMap<ir::RegisterName, RegisterAssign>, usize) {
    let active_info = register_usage.active_info(&control_flow_graph);
    let variables_active_blocks: HashMap<_, _> = register_usage
        .registers()
        .into_iter()
        .map(|it| (it.clone(), active_info.register_active_blocks(it)))
        .collect();
    let mut register_groups = register_groups(
        ir_code,
        ctx,
        &register_usage,
        active_info,
        &variables_active_blocks,
    );
    register_groups.sort_by_cached_key(|group| {
        active_block_intersection(group, &variables_active_blocks).len()
    });
    let mut register_assign = HashMap::new();
    let mut current_used_stack_space = 0;
    let mut next_temporary_register_id = 2;
    for group in register_groups {
        let sample_register = group.iter().next().unwrap();

        let data_type = register_usage.get(sample_register).data_type();
        let type_bytes = (data_type.size(ctx) + 7) / 8;
        let need_registers = type_bytes / 4;
        let assigned_to_register = if next_temporary_register_id + need_registers - 1 <= 6 {
            next_temporary_register_id += need_registers;
            if need_registers == 1 {
                RegisterAssign::Register(format!("t{}", next_temporary_register_id))
            } else {
                RegisterAssign::MultipleRegisters(
                    (next_temporary_register_id..next_temporary_register_id + need_registers)
                        .map(|it| format!("t{}", it))
                        .collect(),
                )
            }
        } else {
            current_used_stack_space += type_bytes;
            RegisterAssign::StackValue(current_used_stack_space)
        };

        for register in group {
            register_assign.insert(register.clone(), assigned_to_register.clone());
        }
    }
    (register_assign, current_used_stack_space)
}

fn active_block_intersection(
    register_group: &HashSet<RegisterName>,
    variables_active_blocks: &HashMap<ir::RegisterName, HashSet<usize>>,
) -> HashSet<usize> {
    register_group
        .iter()
        .map(|it| variables_active_blocks.get(it).unwrap())
        .fold(HashSet::new(), |mut acc, x| {
            acc.extend(x);
            acc
        })
}

fn register_groups(
    ir_code: &ir::FunctionDefinition,
    ctx: &Context,
    register_usage: &RegisterUsageAnalyzer,
    active_info: &ir::analyzer::register_usage::RegisterActiveInfo,
    variables_active_blocks: &HashMap<ir::RegisterName, HashSet<usize>>,
) -> Vec<HashSet<ir::RegisterName>> {
    // todo: collect_phied_registers result can also be mergered
    let mut register_groups = collect_phied_registers(ir_code);
    'a: for register in register_usage.registers() {
        for register_group in register_groups.iter() {
            if register_group.contains(register) {
                continue 'a;
            }
        }
        let data_type = register_usage.get(register).data_type();
        let type_bytes = (data_type.size(ctx) + 7) / 8;
        let need_registers = type_bytes / 4;

        if need_registers <= 1 {
            let register_active_block = active_info.register_active_blocks(register);
            for register_group in register_groups.iter_mut() {
                let register_group_active_blocks =
                    active_block_intersection(register_group, &variables_active_blocks);
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
