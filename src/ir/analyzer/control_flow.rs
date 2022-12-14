use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
};

use bimap::BiMap;
use itertools::Itertools;
use petgraph::{
    algo::{self, dominators::simple_fast},
    prelude::*,
};

use crate::{
    ir::{statement::IRStatement, FunctionDefinition},
    utility,
};

/// [`ControlFlowGraph`] is the control flow graph and related infomation of a function.
#[derive(Debug)]
pub struct ControlFlowGraph {
    graph: DiGraph<(), (), usize>,
    frontiers: HashMap<usize, Vec<usize>>,
    bb_name_index_map: BiMap<usize, String>,
    from_to_may_pass_blocks: RefCell<HashMap<(usize, usize), Vec<usize>>>,
}

impl ControlFlowGraph {
    /// Create a [`ControlFlowGraph`] from a [`FunctionDefinition`].
    pub fn new(function_definition: &FunctionDefinition) -> Self {
        let mut graph = DiGraph::<(), (), usize>::default();
        let bb_name_index_map: BiMap<_, _> = function_definition
            .content
            .iter()
            .enumerate()
            .map(|(index, bb)| (index, bb.name.as_ref().unwrap().clone()))
            .collect();
        for (bb_index, bb) in function_definition.content.iter().enumerate() {
            let last_statement = bb.content.last().unwrap();
            match last_statement {
                IRStatement::Branch(branch) => {
                    let success_node_index = *bb_name_index_map
                        .get_by_right(&branch.success_label)
                        .unwrap();
                    let failure_node_index = *bb_name_index_map
                        .get_by_right(&branch.failure_label)
                        .unwrap();
                    graph.extend_with_edges([
                        (bb_index, success_node_index),
                        (bb_index, failure_node_index),
                    ]);
                }
                IRStatement::Jump(jump) => {
                    let to_node_index = *bb_name_index_map.get_by_right(&jump.label).unwrap();
                    graph.extend_with_edges([(bb_index, to_node_index)]);
                }
                IRStatement::Ret(_) => {
                    graph.extend_with_edges([(bb_index, function_definition.content.len())]);
                }
                _ => unreachable!(),
            }
        }
        let dorminators = simple_fast(&graph, 0.into());
        let graph = remove_unreachable_nodes(graph);
        let frontiers = utility::graph::dominance_frontiers(&dorminators, &graph)
            .into_iter()
            .map(|(k, v)| (k.index(), v.into_iter().map(NodeIndex::index).collect()))
            .collect();
        Self {
            graph,
            frontiers,
            bb_name_index_map,
            from_to_may_pass_blocks: RefCell::new(HashMap::new()),
        }
    }

    /// [Dorminance Frontier](https://en.wikipedia.org/wiki/Dominator_(graph_theory)) of basic block indexed by `bb_index`.
    pub fn dominance_frontier(&self, bb_index: usize) -> &[usize] {
        self.frontiers.get(&bb_index).unwrap()
    }

    /// Get the index of basic block named `name`.
    pub fn basic_block_index_by_name(&self, name: &str) -> usize {
        *self.bb_name_index_map.get_by_right(name).unwrap()
    }

    /// Get the name of basic block indexed by `index`.
    pub fn basic_block_name_by_index(&self, index: usize) -> &str {
        self.bb_name_index_map.get_by_left(&index).unwrap()
    }

    /// Get all blocks that the control flow may pass from `from` to `to`.
    pub fn may_pass_blocks(&self, from: usize, to: usize) -> Ref<Vec<usize>> {
        let mut from_to_passed_blocks = self.from_to_may_pass_blocks.borrow_mut();
        from_to_passed_blocks.entry((from, to)).or_insert_with(|| {
            let mut passed_nodes =
                algo::all_simple_paths::<Vec<_>, _>(&self.graph, from.into(), to.into(), 0, None)
                    .flatten()
                    .map(|it| it.index())
                    .collect_vec();
            passed_nodes.sort();
            passed_nodes.dedup();
            passed_nodes
        });
        drop(from_to_passed_blocks);
        Ref::map(self.from_to_may_pass_blocks.borrow(), |it| {
            it.get(&(from, to)).unwrap()
        })
    }
}

/// Remove unreachable nodes from a graph.
fn remove_unreachable_nodes(mut graph: DiGraph<(), (), usize>) -> DiGraph<(), (), usize> {
    let mut reachable_nodes = vec![];
    // We start from the node indexed by 0, which represents the entry node for functions.
    let mut dfs = Dfs::new(&graph, 0.into());
    while let Some(node) = dfs.next(&graph) {
        reachable_nodes.push(node);
    }
    graph.retain_nodes(|_, it| reachable_nodes.contains(&it));
    graph
}
