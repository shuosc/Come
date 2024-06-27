use std::collections::HashMap;

use crate::{
    ir::{
        editor::{analyzer, Editor},
        quantity::Quantity,
        statement::{phi::PhiSource, IRStatement, Load, Phi, Store},
        FunctionDefinition, RegisterName,
    },
    utility::data_type::Type,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::{
    remove_only_once_store::RemoveOnlyOnceStore, remove_unused_register::RemoveUnusedRegister,
    IsPass,
};

/// [`MemoryToRegister`] is a pass that convert memory access to register access.
/// It is similar to LLVM's [`mem2reg`](https://llvm.org/docs/Passes.html#mem2reg-promote-memory-to-register).
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct MemoryToRegister;

impl IsPass for MemoryToRegister {
    fn run(&self, editor: &mut Editor) {
        let insert_phis_at = insert_phi_positions(
            &editor.binded_analyzer().memory_usage(),
            &editor.binded_analyzer().control_flow_graph(),
        );
        // There exists two parts of actions:
        // - The first part will remove the load and store statements, and replace the load targets with the "phi"ed results
        // - The second part will insert the phi nodes
        let (to_renames, to_removes, subnodes) = decide_values(
            &editor.content,
            &editor.binded_analyzer().control_flow_graph(),
            &insert_phis_at,
        );
        editor.remove_statements(to_removes);
        for (from, to) in to_renames {
            editor.rename_local(from, to);
        }
        // let insert_phi_actions = create_phi_node_insertion_actions(
        //     subnodes,
        //     &analyzer.memory_usage.memory_access_variables_and_types(),
        //     analyzer.content,
        // );
        insert_phi_nodes(subnodes, editor);
    }

    fn need(&self) -> Vec<super::Pass> {
        vec![RemoveOnlyOnceStore.into()]
    }

    fn invalidate(&self) -> Vec<super::Pass> {
        vec![RemoveUnusedRegister.into()]
    }
}

/// Find out where should we insert phi positions.
/// Return a vector which contains (VariableName, BasicBlockIndex)
fn insert_phi_positions(
    memory_usage: &analyzer::BindedMemoryUsage,
    control_flow_graph: &analyzer::BindedControlFlowGraph,
) -> Vec<(String, usize)> {
    let mut result = Vec::new();
    for variable_name in memory_usage.memory_access_variables() {
        let memory_access_info = memory_usage.memory_access_info(variable_name);
        // for each store to this variable,
        // we find the dominance_frontier of the basic block it is in
        let mut pending_bb_indexes = memory_access_info.store.iter().map(|it| it.0).collect_vec();
        pending_bb_indexes.dedup();
        let mut done_bb_index = Vec::new();
        while let Some(considering_bb_index) = pending_bb_indexes.pop() {
            done_bb_index.push(considering_bb_index);
            let dominator_frontier_bb_indexes =
                control_flow_graph.dominance_frontier(considering_bb_index);
            for &to_bb_index in dominator_frontier_bb_indexes {
                result.push((variable_name.0.clone(), to_bb_index));
                // it's possible we put a phi node to a new block which contains no
                // store to this variable in the past, in such cases we should look at the bacic block
                // of this phi node too
                let to_bb_had_no_store_to_this_variable = !done_bb_index.contains(&to_bb_index)
                    && !pending_bb_indexes.contains(&to_bb_index);
                if to_bb_had_no_store_to_this_variable {
                    #[cfg(test)]
                    cov_mark::hit!(generated_phi_spread_value);

                    pending_bb_indexes.push(to_bb_index);
                }
            }
        }
    }
    result.sort();
    result.dedup();
    result
}

/// Decide which value should be used for the phi nodes for variable which name is `variable_name`.
fn decide_variable_value(
    variable_name: &str,
    current_variable_value: &[HashMap<String, (usize, Quantity)>],
) -> (usize, Quantity) {
    for frame in current_variable_value.iter().rev() {
        if let Some(value) = frame.get(variable_name) {
            return value.clone();
        }
    }
    unreachable!()
}

// We need to know all incoming "arrows" to a phi node before we can construct it.
// So we created this data structure to store the "unfinished" phi nodes.
struct PhiSubNode {
    basic_block_index: usize,
    variable_name: String,
    value_from: usize,
    value: Quantity,
}

type DecideValueResult = (
    Vec<(RegisterName, Quantity)>,
    Vec<(usize, usize)>,
    Vec<PhiSubNode>,
);

/// Returns (Actions to edit the statements, PhiSubNodes to insert)
fn decide_values_start_from(
    function: &FunctionDefinition,
    control_flow_graph: &analyzer::BindedControlFlowGraph,
    consider_block_index: usize,
    inserted_phi: &[(String, usize)],
    visited: &mut Vec<usize>,
    current_variable_value: &mut Vec<HashMap<String, (usize, Quantity)>>,
) -> DecideValueResult {
    let mut to_rename = Vec::new();
    let mut to_remove = Vec::new();
    let mut subnodes = Vec::new();
    let block = &function[consider_block_index];
    let phied_variables = inserted_phi
        .iter()
        .filter(|(_, bb_id)| bb_id == &consider_block_index)
        .map(|(variable_name, _)| variable_name);
    for variable_name in phied_variables {
        let (from_basic_block_index, value) =
            decide_variable_value(variable_name, current_variable_value);
        subnodes.push(PhiSubNode {
            basic_block_index: consider_block_index,
            variable_name: variable_name.clone(),
            value_from: from_basic_block_index,
            value,
        });
        current_variable_value.last_mut().unwrap().insert(
            variable_name.clone(),
            (
                consider_block_index,
                RegisterName(format!("{}_{}", variable_name, block.name.clone().unwrap())).into(),
            ),
        );
    }
    if visited.contains(&consider_block_index) {
        return (to_rename, to_remove, subnodes);
    }
    visited.push(consider_block_index);
    for (statement_index, statement) in block.content.iter().enumerate() {
        match statement {
            IRStatement::Load(Load {
                to,
                from: Quantity::RegisterName(local),
                ..
            }) => {
                let (_, replace_with_value) =
                    decide_variable_value(&local.0, current_variable_value);
                to_remove.push((consider_block_index, statement_index));
                to_rename.push((to.clone(), replace_with_value));
            }
            IRStatement::Store(Store {
                source,
                target: Quantity::RegisterName(local),
                ..
            }) => {
                current_variable_value
                    .last_mut()
                    .unwrap()
                    .insert(local.0.clone(), (consider_block_index, source.clone()));
                to_remove.push((consider_block_index, statement_index));
            }
            IRStatement::Branch(branch) => {
                let success_block =
                    control_flow_graph.basic_block_index_by_name(&branch.success_label);
                current_variable_value.push(HashMap::new());
                let (mut inner_to_rename, mut inner_to_remove, mut subnodes_on_success) =
                    decide_values_start_from(
                        function,
                        control_flow_graph,
                        success_block,
                        inserted_phi,
                        visited,
                        current_variable_value,
                    );
                subnodes.append(&mut subnodes_on_success);
                to_rename.append(&mut inner_to_rename);
                to_remove.append(&mut inner_to_remove);
                current_variable_value.pop();
                current_variable_value.push(HashMap::new());
                let failure_block =
                    control_flow_graph.basic_block_index_by_name(&branch.failure_label);
                let (mut inner_to_rename, mut inner_to_remove, mut subnodes_on_failure) =
                    decide_values_start_from(
                        function,
                        control_flow_graph,
                        failure_block,
                        inserted_phi,
                        visited,
                        current_variable_value,
                    );
                subnodes.append(&mut subnodes_on_failure);
                to_rename.append(&mut inner_to_rename);
                to_remove.append(&mut inner_to_remove);
                current_variable_value.pop();
            }
            IRStatement::Jump(jump) => {
                let jump_to_block = control_flow_graph.basic_block_index_by_name(&jump.label);
                let (mut inner_to_rename, mut inner_to_remove, mut subnodes_on_jump) =
                    decide_values_start_from(
                        function,
                        control_flow_graph,
                        jump_to_block,
                        inserted_phi,
                        visited,
                        current_variable_value,
                    );
                subnodes.append(&mut subnodes_on_jump);
                to_rename.append(&mut inner_to_rename);
                to_remove.append(&mut inner_to_remove);
            }
            _ => (),
        }
    }
    (to_rename, to_remove, subnodes)
}

fn decide_values(
    function: &FunctionDefinition,
    control_flow_graph: &analyzer::BindedControlFlowGraph,
    inserted_phi: &[(String, usize)],
) -> DecideValueResult {
    let mut visited = Vec::new();
    let mut current_variable_value = vec![HashMap::new()];
    decide_values_start_from(
        function,
        control_flow_graph,
        0,
        inserted_phi,
        &mut visited,
        &mut current_variable_value,
    )
}

fn insert_phi_nodes(mut subnodes: Vec<PhiSubNode>, editor: &mut Editor) {
    subnodes
        .sort_unstable_by_key(|subnode| (subnode.basic_block_index, subnode.variable_name.clone()));
    subnodes
        .into_iter()
        .group_by(|subnode| subnode.basic_block_index)
        .into_iter()
        .flat_map(|(basicblock_index, subnodes_in_basicblock)| {
            subnodes_in_basicblock
                .into_iter()
                .group_by(|subnode| subnode.variable_name.clone())
                .into_iter()
                .map(|(variable_name, subnodes_for_variable)| {
                    let phi_node = create_phi_node(
                        &variable_name,
                        basicblock_index,
                        subnodes_for_variable,
                        &editor
                            .binded_analyzer()
                            .memory_usage()
                            .memory_access_variables_and_types(),
                        &editor.content,
                    );
                    editor.push_front_statement(basicblock_index, phi_node)
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn create_phi_node(
    variable_name: &str,
    basic_block_index: usize,
    subnodes_for_variable: impl IntoIterator<Item = PhiSubNode>,
    memory_access_variables_and_types: &HashMap<RegisterName, Type>,
    function: &FunctionDefinition,
) -> Phi {
    let from_basic_block_name = function[basic_block_index].name.clone().unwrap();
    let register_name = RegisterName(format!("{variable_name}_{from_basic_block_name}"));
    let data_type = memory_access_variables_and_types
        .get(&RegisterName(variable_name.to_string()))
        .unwrap()
        .clone();
    let from = subnodes_for_variable
        .into_iter()
        .map(|entry| {
            let from_basic_block_name = function[entry.value_from].name.clone().unwrap();
            PhiSource {
                value: entry.value,
                block: from_basic_block_name,
            }
        })
        .collect();
    Phi {
        to: register_name,
        data_type,
        from,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::borrow_interior_mutable_const)]

    use super::*;
    use crate::{
        ir::{
            self,
            function::{basic_block::BasicBlock, test_util::*},
            statement::Ret,
        },
        utility::data_type,
    };

    #[test]
    fn simple() {
        let function_definition = FunctionDefinition {
            header: ir::FunctionHeader {
                name: "f".to_string(),
                parameters: Vec::new(),
                return_type: data_type::I32.clone(),
            },
            content: vec![
                BasicBlock {
                    name: None,
                    content: vec![
                        alloca("a"),
                        alloca("b"),
                        alloca("c"),
                        store("a"),
                        store("b"),
                        store("c"),
                        branch("bb1", "bb2"),
                    ],
                },
                BasicBlock {
                    name: Some("bb1".to_string()),
                    content: vec![
                        load("a", 0),
                        load("b", 0),
                        binop("t_0", "a_0", "b_0"),
                        store_with_reg("c", "t_0"),
                        jump("bb3"),
                    ],
                },
                BasicBlock {
                    name: Some("bb2".to_string()),
                    content: vec![
                        load("a", 1),
                        load("b", 1),
                        binop("t_1", "a_1", "b_1"),
                        store_with_reg("c", "t_1"),
                        jump("bb3"),
                    ],
                },
                BasicBlock {
                    name: Some("bb3".to_string()),
                    content: vec![
                        load("c", 0),
                        Ret {
                            value: Some(RegisterName("c_0".to_string()).into()),
                        }
                        .into(),
                    ],
                },
            ],
        };

        let mut editor = Editor::new(function_definition);
        let pass = MemoryToRegister;
        pass.run(&mut editor);

        let the_phi_statement = editor.content[3].content[0].as_phi();
        assert_eq!(the_phi_statement.to, RegisterName("c_addr_bb3".to_string()));
        assert_eq!(the_phi_statement.from.len(), 2);
        assert!(the_phi_statement.from.contains(&PhiSource {
            value: RegisterName("t_0".to_string()).into(),
            block: "bb1".to_string()
        }));
        assert!(the_phi_statement.from.contains(&PhiSource {
            value: RegisterName("t_1".to_string()).into(),
            block: "bb2".to_string()
        }));
    }

    #[test]
    fn not_storing_unused() {
        let function_definition = FunctionDefinition {
            header: ir::FunctionHeader {
                name: "f".to_string(),
                parameters: Vec::new(),
                return_type: data_type::I32.clone(),
            },
            content: vec![
                BasicBlock {
                    name: None,
                    content: vec![
                        alloca("a"),
                        alloca("b"),
                        alloca("c"),
                        store("a"),
                        store("b"),
                        // we don't store c here
                        branch("bb1", "bb2"),
                    ],
                },
                BasicBlock {
                    name: Some("bb1".to_string()),
                    content: vec![
                        load("a", 0),
                        load("b", 0),
                        binop("t_0", "a_0", "b_0"),
                        store_with_reg("c", "t_0"),
                        jump("bb3"),
                    ],
                },
                BasicBlock {
                    name: Some("bb2".to_string()),
                    content: vec![
                        load("a", 1),
                        load("b", 1),
                        binop("t_1", "a_1", "b_1"),
                        store_with_reg("c", "t_1"),
                        jump("bb3"),
                    ],
                },
                BasicBlock {
                    name: Some("bb3".to_string()),
                    content: vec![
                        load("c", 0),
                        Ret {
                            value: Some(RegisterName("c_0".to_string()).into()),
                        }
                        .into(),
                    ],
                },
            ],
        };
        let mut editor = Editor::new(function_definition);
        let pass = MemoryToRegister;
        pass.run(&mut editor);
        let the_phi_statement = editor.content[3].content[0].as_phi();
        assert_eq!(the_phi_statement.to, RegisterName("c_addr_bb3".to_string()));
        assert_eq!(the_phi_statement.from.len(), 2);
        assert!(the_phi_statement.from.contains(&PhiSource {
            value: RegisterName("t_0".to_string()).into(),
            block: "bb1".to_string()
        }));
        assert!(the_phi_statement.from.contains(&PhiSource {
            value: RegisterName("t_1".to_string()).into(),
            block: "bb2".to_string()
        }));
    }

    #[test]
    fn remove_load_in_multiple_basic_blocks() {
        let function_definition = FunctionDefinition {
            header: ir::FunctionHeader {
                name: "f".to_string(),
                parameters: Vec::new(),
                return_type: data_type::I32.clone(),
            },
            content: vec![
                BasicBlock {
                    name: None,
                    content: vec![
                        alloca("a"),
                        alloca("b"),
                        alloca("c"),
                        store("a"),
                        store("b"),
                        store("c"),
                        branch("bb1", "bb2"),
                    ],
                },
                BasicBlock {
                    name: Some("bb1".to_string()),
                    content: vec![
                        load("a", 0),
                        load("b", 0),
                        binop("t_0", "a_0", "b_0"),
                        store_with_reg("c", "t_0"),
                        jump("bb4"),
                    ],
                },
                BasicBlock {
                    name: Some("bb2".to_string()),
                    content: vec![
                        load("a", 1),
                        load("b", 1),
                        binop("t_1", "a_1", "b_1"),
                        store_with_reg("c", "t_1"),
                        branch("bb3", "bb4"),
                    ],
                },
                BasicBlock {
                    name: Some("bb3".to_string()),
                    content: vec![
                        load("a", 2),
                        load("b", 2),
                        load("c", 0),
                        binop("t_2", "a_2", "b_2"),
                        binop("t_3", "t_2", "c_0"),
                        store_with_reg("c", "t_3"),
                        jump("bb4"),
                    ],
                },
                BasicBlock {
                    name: Some("bb4".to_string()),
                    content: vec![
                        load("a", 3),
                        load("b", 3),
                        store("a"),
                        store("b"),
                        load("a", 4),
                        load("b", 4),
                        binop("t_4", "a_4", "b_4"),
                        load("c", 1),
                        binop("t_5", "t_4", "c_1"),
                        Ret {
                            value: Some(RegisterName("t_5".to_string()).into()),
                        }
                        .into(),
                    ],
                },
            ],
        };
        let mut editor = Editor::new(function_definition);
        let pass = MemoryToRegister;
        pass.run(&mut editor);
        let generated_phi = editor.content[4].content[0].as_phi();
        assert_eq!(generated_phi.to, RegisterName("c_addr_bb4".to_string()));
        assert_eq!(generated_phi.from.len(), 3);
        assert!(generated_phi.from.contains(&PhiSource {
            value: RegisterName("t_3".to_string()).into(),
            block: "bb3".to_string()
        }));
        assert!(generated_phi.from.contains(&PhiSource {
            value: RegisterName("t_1".to_string()).into(),
            block: "bb2".to_string()
        }));
        assert!(generated_phi.from.contains(&PhiSource {
            value: RegisterName("t_0".to_string()).into(),
            block: "bb1".to_string()
        }));
    }

    #[test]
    fn generate_store_like_phi() {
        let function_definition = FunctionDefinition {
            header: ir::FunctionHeader {
                name: "f".to_string(),
                parameters: Vec::new(),
                return_type: data_type::I32.clone(),
            },
            content: vec![
                BasicBlock {
                    name: None,
                    content: vec![alloca("a"), store("a"), branch("bb2", "bb5")],
                },
                BasicBlock {
                    name: Some("bb2".to_string()),
                    content: vec![branch("bb3", "bb4")],
                },
                BasicBlock {
                    name: Some("bb3".to_string()),
                    content: vec![load("a", 0), store("a"), jump("bb7")],
                },
                BasicBlock {
                    name: Some("bb4".to_string()),
                    content: vec![load("a", 1), store("a"), jump("bb7")],
                },
                BasicBlock {
                    name: Some("bb7".to_string()),
                    content: vec![load("a", 2), jump("bb6")],
                },
                BasicBlock {
                    name: Some("bb5".to_string()),
                    content: vec![
                        load("a", 3),
                        binop("t_0", "a_3", "a_3"),
                        store_with_reg("a", "t_0"),
                        jump("bb6"),
                    ],
                },
                BasicBlock {
                    name: Some("bb6".to_string()),
                    content: vec![
                        load("a", 4),
                        Ret {
                            value: Some(RegisterName("a_4".to_string()).into()),
                        }
                        .into(),
                    ],
                },
            ],
        };
        cov_mark::check!(generated_phi_spread_value);
        let mut editor = Editor::new(function_definition);
        let pass = MemoryToRegister;
        pass.run(&mut editor);
        let generated_phi = editor.content[4].content[0].as_phi();
        assert_eq!(generated_phi.to, RegisterName("a_addr_bb7".to_string()));
        assert_eq!(generated_phi.from.len(), 2);
        assert!(generated_phi.from.contains(&PhiSource {
            value: 1.into(),
            block: "bb4".to_string()
        }));
        assert!(generated_phi.from.contains(&PhiSource {
            value: 1.into(),
            block: "bb3".to_string()
        }));

        let generated_phi = editor.content[6].content[0].as_phi();
        assert_eq!(generated_phi.to, RegisterName("a_addr_bb6".to_string()));
        assert_eq!(generated_phi.from.len(), 2);
        assert!(generated_phi.from.contains(&PhiSource {
            value: RegisterName("t_0".to_string()).into(),
            block: "bb5".to_string()
        }));
        assert!(generated_phi.from.contains(&PhiSource {
            value: RegisterName("a_addr_bb7".to_string()).into(),
            block: "bb7".to_string()
        }));
    }

    #[test]
    fn self_refrence_phi() {
        let function_definition = FunctionDefinition {
            header: ir::FunctionHeader {
                name: "f".to_string(),
                parameters: Vec::new(),
                return_type: data_type::I32.clone(),
            },
            content: vec![
                BasicBlock {
                    name: None,
                    content: vec![alloca("a"), store("a"), branch("bb2", "bb5")],
                },
                BasicBlock {
                    name: Some("bb2".to_string()),
                    content: vec![branch("bb3", "bb4")],
                },
                BasicBlock {
                    name: Some("bb3".to_string()),
                    content: vec![load("a", 0), store("a"), jump("bb7")],
                },
                BasicBlock {
                    name: Some("bb4".to_string()),
                    content: vec![load("a", 1), store("a"), jump("bb7")],
                },
                BasicBlock {
                    name: Some("bb7".to_string()),
                    content: vec![load("a", 2), jump("bb6")],
                },
                BasicBlock {
                    name: Some("bb5".to_string()),
                    content: vec![
                        load("a", 3),
                        binop("t_0", "a_3", "a_3"),
                        store_with_reg("a", "t_0"),
                        branch("bb6", "bb5"),
                    ],
                },
                BasicBlock {
                    name: Some("bb6".to_string()),
                    content: vec![
                        load("a", 4),
                        Ret {
                            value: Some(RegisterName("a_4".to_string()).into()),
                        }
                        .into(),
                    ],
                },
            ],
        };
        let mut editor = Editor::new(function_definition);
        let pass = MemoryToRegister;
        pass.run(&mut editor);
        let generated_phi = editor.content[4].content[0].as_phi();
        assert_eq!(generated_phi.to, RegisterName("a_addr_bb7".to_string()));
        assert_eq!(generated_phi.from.len(), 2);
        assert!(generated_phi.from.contains(&PhiSource {
            value: 1.into(),
            block: "bb3".to_string()
        }));
        assert!(generated_phi.from.contains(&PhiSource {
            value: 1.into(),
            block: "bb4".to_string()
        }));

        let generated_phi = editor.content[5].content[0].as_phi();
        assert_eq!(generated_phi.to, RegisterName("a_addr_bb5".to_string()));
        assert_eq!(generated_phi.from.len(), 2);
        assert!(generated_phi.from.contains(&PhiSource {
            value: 1.into(),
            block: "f_entry".to_string()
        }));
        assert!(generated_phi.from.contains(&PhiSource {
            value: RegisterName("t_0".to_string()).into(),
            block: "bb5".to_string()
        }));

        let generated_phi = editor.content[6].content[0].as_phi();
        assert_eq!(generated_phi.to, RegisterName("a_addr_bb6".to_string()));
        assert_eq!(generated_phi.from.len(), 2);
        assert!(generated_phi.from.contains(&PhiSource {
            value: RegisterName("a_addr_bb7".to_string()).into(),
            block: "bb7".to_string()
        }));
        assert!(generated_phi.from.contains(&PhiSource {
            value: RegisterName("t_0".to_string()).into(),
            block: "bb5".to_string()
        }));
    }

    #[test]
    fn comprehensive() {
        let function_definition = FunctionDefinition {
            header: ir::FunctionHeader {
                name: "f".to_string(),
                parameters: Vec::new(),
                return_type: data_type::I32.clone(),
            },
            content: vec![
                BasicBlock {
                    name: None,
                    content: vec![alloca("a"), store("a"), branch("bb2", "bb4")],
                },
                BasicBlock {
                    name: Some("bb2".to_string()),
                    content: vec![
                        load("a", 0),
                        binop("t_0", "a_0", "a_0"),
                        store_with_reg("a", "t_0"),
                        jump("bb3"),
                    ],
                },
                BasicBlock {
                    name: Some("bb3".to_string()),
                    content: vec![
                        load("a", 1),
                        binop("t_1", "a_1", "a_1"),
                        store_with_reg("a", "t_1"),
                        branch("bb2", "bb8"),
                    ],
                },
                BasicBlock {
                    name: Some("bb4".to_string()),
                    content: vec![branch("bb5", "bb6")],
                },
                BasicBlock {
                    name: Some("bb5".to_string()),
                    content: vec![load("a", 2), store("a"), jump("bb7")],
                },
                BasicBlock {
                    name: Some("bb6".to_string()),
                    content: vec![load("a", 3), store("a"), jump("bb7")],
                },
                BasicBlock {
                    name: Some("bb7".to_string()),
                    content: vec![load("a", 4), jump("bb8")],
                },
                BasicBlock {
                    name: Some("bb8".to_string()),
                    content: vec![
                        load("a", 5),
                        Ret {
                            value: Some(RegisterName("a_5".to_string()).into()),
                        }
                        .into(),
                    ],
                },
            ],
        };
        let mut editor = Editor::new(function_definition);
        let pass = MemoryToRegister;
        pass.run(&mut editor);
        let generated_phi = editor.content[1].content[0].as_phi();
        assert_eq!(generated_phi.to, RegisterName("a_addr_bb2".to_string()));
        assert_eq!(generated_phi.from.len(), 2);
        assert!(generated_phi.from.contains(&PhiSource {
            value: 1.into(),
            block: "f_entry".to_string()
        }));
        assert!(generated_phi.from.contains(&PhiSource {
            value: RegisterName("t_1".to_string()).into(),
            block: "bb3".to_string()
        }));

        let generated_phi = editor.content[6].content[0].as_phi();
        assert_eq!(generated_phi.to, RegisterName("a_addr_bb7".to_string()));
        assert_eq!(generated_phi.from.len(), 2);
        assert!(generated_phi.from.contains(&PhiSource {
            value: 1.into(),
            block: "bb5".to_string()
        }));
        assert!(generated_phi.from.contains(&PhiSource {
            value: 1.into(),
            block: "bb6".to_string()
        }));

        let generated_phi = editor.content[7].content[0].as_phi();
        assert_eq!(generated_phi.to, RegisterName("a_addr_bb8".to_string()));
        assert_eq!(generated_phi.from.len(), 2);
        assert!(generated_phi.from.contains(&PhiSource {
            value: RegisterName("t_1".to_string()).into(),
            block: "bb3".to_string()
        }));
        assert!(generated_phi.from.contains(&PhiSource {
            value: RegisterName("a_addr_bb7".to_string()).into(),
            block: "bb7".to_string()
        }));
    }
}
