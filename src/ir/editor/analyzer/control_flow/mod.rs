mod control_flow_loop;
use std::{
    cell::{OnceCell, Ref, RefCell},
    collections::HashMap,
};

use bimap::BiMap;
use itertools::Itertools;
use petgraph::{
    algo::{
        self,
        dominators::{simple_fast, Dominators},
    },
    prelude::*,
};

use crate::{
    ir::{self, editor::action::Action, statement::IRStatement, FunctionDefinition},
    utility::{self},
};

use super::IsAnalyzer;
pub use control_flow_loop::{Scc, SccContent};

mod scc_new;
pub use scc_new::BindedScc;

/// [`ControlFlowGraph`] is the control flow graph and related infomation of a function.
#[derive(Debug)]
pub struct ControlFlowGraphContent {
    graph: DiGraph<(), (), usize>,
    frontiers: HashMap<usize, Vec<usize>>,
    bb_name_index_map: BiMap<usize, String>,
    dominators: Dominators<NodeIndex<usize>>,
    // fixme: remove this refcell!
    from_to_may_pass_blocks: RefCell<HashMap<(usize, usize), Vec<usize>>>,
}

impl ControlFlowGraphContent {
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
            dominators: dorminators,
            bb_name_index_map,
            from_to_may_pass_blocks: RefCell::new(HashMap::new()),
        }
    }

    /// [Dorminance Frontier](https://en.wikipedia.org/wiki/Dominator_(graph_theory)) of basic block indexed by `bb_index`.
    pub fn dominance_frontier(&self, bb_index: usize) -> &[usize] {
        self.frontiers.get(&bb_index).unwrap()
    }

    fn dominates_calculate(&self, visiting: usize, visited: &mut Vec<usize>) {
        if visited.contains(&visiting) {
            return;
        }
        visited.push(visiting);
        let mut imm_dominates: Vec<usize> = self.immediately_dominates(visiting);
        imm_dominates.retain(|it| !visited.contains(it));
        for it in imm_dominates {
            self.dominates_calculate(it, visited);
        }
    }

    fn immediately_dominates(&self, node: usize) -> Vec<usize> {
        self.dominators
            .immediately_dominated_by(node.into())
            .map(|it| it.index())
            .collect()
    }

    pub fn dominates(&self, node: usize) -> Vec<usize> {
        let mut visited = Vec::new();
        self.dominates_calculate(node, &mut visited);
        visited
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

#[derive(Default, Debug)]
pub struct ControlFlowGraph(OnceCell<ControlFlowGraphContent>);
impl ControlFlowGraph {
    pub fn new() -> Self {
        Self(OnceCell::new())
    }
    fn content(&self, content: &FunctionDefinition) -> &ControlFlowGraphContent {
        self.0.get_or_init(|| ControlFlowGraphContent::new(content))
    }
    fn dominance_frontier(&self, content: &ir::FunctionDefinition, bb_index: usize) -> &[usize] {
        self.content(content).dominance_frontier(bb_index)
    }
    fn basic_block_index_by_name(&self, content: &ir::FunctionDefinition, name: &str) -> usize {
        self.content(content).basic_block_index_by_name(name)
    }
    fn basic_block_name_by_index(&self, content: &ir::FunctionDefinition, index: usize) -> &str {
        self.content(content).basic_block_name_by_index(index)
    }
    fn may_pass_blocks(
        &self,
        content: &ir::FunctionDefinition,
        from: usize,
        to: usize,
    ) -> Ref<Vec<usize>> {
        self.content(content).may_pass_blocks(from, to)
    }
    fn dominate(&self, content: &ir::FunctionDefinition, bb_index: usize) -> Vec<usize> {
        self.content(content).dominates(bb_index)
    }
    // todo: cache it
    fn sccs(&self, content: &FunctionDefinition) -> Scc {
        let graph = &self.content(content).graph;
        let nodes: Vec<_> = graph.node_indices().collect();
        Scc::new(graph, &nodes, &[])
    }
}

pub struct BindedControlFlowGraph<'item, 'bind: 'item> {
    pub bind_on: &'bind FunctionDefinition,
    item: &'item ControlFlowGraph,
}

impl<'item, 'bind: 'item> BindedControlFlowGraph<'item, 'bind> {
    pub fn dominance_frontier(&self, bb_index: usize) -> &[usize] {
        self.item.dominance_frontier(self.bind_on, bb_index)
    }
    pub fn basic_block_index_by_name(&self, name: &str) -> usize {
        self.item.basic_block_index_by_name(self.bind_on, name)
    }
    pub fn basic_block_name_by_index(&self, index: usize) -> &str {
        self.item.basic_block_name_by_index(self.bind_on, index)
    }
    pub fn may_pass_blocks(&self, from: usize, to: usize) -> Ref<Vec<usize>> {
        self.item.may_pass_blocks(self.bind_on, from, to)
    }
    pub fn sccs(&self) -> Scc {
        self.item.sccs(self.bind_on)
    }
    pub fn graph(&self) -> &DiGraph<(), (), usize> {
        &self.item.content(self.bind_on).graph
    }
    pub fn dominates(&self, bb_index: usize) -> Vec<usize> {
        self.item.dominate(self.bind_on, bb_index)
    }
    pub fn predecessor(&self, bb_index: usize) -> Vec<usize> {
        self.graph()
            .neighbors_directed(bb_index.into(), Direction::Incoming)
            .map(|it| it.index())
            .collect()
    }

    pub fn successors(&self, bb_index: usize) -> Vec<usize> {
        self.graph()
            .neighbors_directed(bb_index.into(), Direction::Incoming)
            .map(|it| it.index())
            .collect()
    }

    pub fn not_dominate_successors(&self, bb_index: usize) -> Vec<usize> {
        let successors = self
            .graph()
            .neighbors_directed(bb_index.into(), Direction::Incoming)
            .map(|it| it.index());
        let nodes_dominated = self.dominates(bb_index);
        successors
            .filter(|it| !nodes_dominated.contains(it))
            .collect()
    }

    pub fn scc_new(&self) -> BindedScc<'_> {
        let graph = &self.item.content(self.bind_on).graph;
        let nodes = graph.node_indices().map(|it| it.index()).collect_vec();
        BindedScc::new(graph, nodes.into_iter(), true)
    }
}

impl<'item, 'bind: 'item> IsAnalyzer<'item, 'bind> for ControlFlowGraph {
    type Binded = BindedControlFlowGraph<'item, 'bind>;

    fn on_action(&mut self, _action: &Action) {
        self.0.take();
    }

    fn bind(&'item self, content: &'bind ir::FunctionDefinition) -> Self::Binded {
        BindedControlFlowGraph {
            bind_on: content,
            item: self,
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ir::{
            function::{basic_block::BasicBlock, test_util::*},
            statement::Ret,
        },
        utility::data_type,
    };

    #[test]
    fn test_loop() {
        let control_flow_graph = ControlFlowGraph::new();
        let function_definition = FunctionDefinition {
            header: ir::FunctionHeader {
                name: "f".to_string(),
                parameters: Vec::new(),
                return_type: data_type::Type::None,
            },
            content: vec![
                BasicBlock {
                    name: Some("bb0".to_string()),
                    content: vec![branch("bb1", "bb2")],
                },
                BasicBlock {
                    name: Some("bb1".to_string()),
                    content: vec![jump("bb3")],
                },
                BasicBlock {
                    name: Some("bb2".to_string()),
                    content: vec![jump("bb6")],
                },
                BasicBlock {
                    name: Some("bb3".to_string()),
                    content: vec![jump("bb4")],
                },
                BasicBlock {
                    name: Some("bb4".to_string()),
                    content: vec![branch("bb5", "bb9")],
                },
                BasicBlock {
                    name: Some("bb5".to_string()),
                    content: vec![branch("bb1", "bb3")],
                },
                BasicBlock {
                    name: Some("bb6".to_string()),
                    content: vec![branch("bb7", "bb8")],
                },
                BasicBlock {
                    name: Some("bb7".to_string()),
                    content: vec![jump("bb2")],
                },
                BasicBlock {
                    name: Some("bb8".to_string()),
                    content: vec![branch("bb7", "bb9")],
                },
                BasicBlock {
                    name: Some("bb9".to_string()),
                    content: vec![Ret { value: None }.into()],
                },
            ],
        };
        let loops = control_flow_graph.bind(&function_definition).sccs();
        assert!(loops.content.contains(&SccContent::Node(0)));
        assert!(loops.content.contains(&SccContent::Node(9)));
        assert!(loops
            .content
            .iter()
            .any(|it| if let SccContent::SubScc(subloop) = it {
                subloop.entries.contains(&1)
            } else {
                false
            }));
        assert!(loops
            .content
            .iter()
            .any(|it| if let SccContent::SubScc(subloop) = it {
                subloop.entries.contains(&2)
            } else {
                false
            }));
    }
}
