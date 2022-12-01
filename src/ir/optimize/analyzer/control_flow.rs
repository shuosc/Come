use std::{cell::RefCell, collections::HashMap};

use bimap::BiMap;
use petgraph::{
    algo::dominators::{simple_fast, Dominators},
    prelude::*,
    visit::GraphBase,
};

use crate::{
    ir::{statement::IRStatement, FunctionDefinition},
    utility::{self},
};

type NodeId = <DiGraph<usize, ()> as GraphBase>::NodeId;

#[derive(Debug)]
pub struct ControlFlowGraph {
    graph: DiGraph<usize, ()>,
    dorminators: Dominators<NodeId>,
    frontiers: HashMap<NodeId, Vec<NodeId>>,
    bb_name_index_map: BiMap<String, usize>,
    bb_index_node_index_map: BiMap<usize, NodeId>,
    bb_index_dfs_index_map: RefCell<BiMap<usize, usize>>,
    start_node: NodeId,
    end_node: NodeId,
}

impl ControlFlowGraph {
    pub fn new(function_definition: &FunctionDefinition) -> Self {
        let mut graph = DiGraph::<usize, ()>::new();
        let mut bb_index_node_index_map = BiMap::new();
        let start_node = graph.add_node(0);
        let mut bb_name_index_map = BiMap::new();
        let mut first_node = None;
        for (bb_index, bb) in function_definition.content.iter().enumerate() {
            let bb_node = graph.add_node(bb_index + 1);
            if first_node.is_none() {
                first_node = Some(bb_node);
            }
            bb_index_node_index_map.insert(bb_index, bb_node);
            bb_name_index_map.insert(bb.name.clone().unwrap(), bb_index);
        }
        let end_node = graph.add_node(usize::MAX);
        graph.add_edge(start_node, first_node.unwrap(), ());
        for (bb_index, bb) in function_definition.content.iter().enumerate() {
            if let Some(last_statement) = bb.content.last() {
                let bb_node_index = bb_index_node_index_map.get_by_left(&bb_index).unwrap();
                match last_statement {
                    IRStatement::Branch(branch) => {
                        let success_node_index = *bb_index_node_index_map
                            .get_by_left(
                                bb_name_index_map
                                    .get_by_left(&branch.success_label.clone())
                                    .unwrap(),
                            )
                            .unwrap();
                        graph.add_edge(*bb_node_index, success_node_index, ());
                        let failure_node_index = *bb_name_index_map
                            .get_by_left(&branch.failure_label.clone())
                            .map(|bb_index| bb_index_node_index_map.get_by_left(bb_index).unwrap())
                            .unwrap();
                        graph.add_edge(*bb_node_index, failure_node_index, ());
                    }
                    IRStatement::Jump(jump) => {
                        let to_node_index = *bb_index_node_index_map
                            .get_by_left(
                                bb_name_index_map.get_by_left(&jump.label.clone()).unwrap(),
                            )
                            .unwrap();
                        graph.add_edge(*bb_node_index, to_node_index, ());
                    }
                    IRStatement::Ret(_) => {
                        graph.add_edge(*bb_node_index, end_node, ());
                    }
                    _ => unreachable!(),
                }
            }
        }
        let dorminators = simple_fast(&graph, start_node);
        let frontiers = utility::graph::dominance_frontiers(&dorminators, &graph);
        Self {
            graph,
            dorminators,
            frontiers,
            bb_index_node_index_map,
            start_node,
            end_node,
            bb_name_index_map,
            bb_index_dfs_index_map: RefCell::new(BiMap::new()),
        }
    }

    pub fn dorminate_frontier(&self, bb_index: usize) -> Vec<usize> {
        let node = self.bb_index_node_index_map.get_by_left(&bb_index).unwrap();
        self.frontiers
            .get(node)
            .unwrap()
            .iter()
            .map(|node| *self.bb_index_node_index_map.get_by_right(node).unwrap())
            .collect()
    }

    pub fn basic_block_index_by_name(&self, name: &str) -> usize {
        *self.bb_name_index_map.get_by_left(name).unwrap()
    }

    pub fn basic_block_name_by_index(&self, index: usize) -> &String {
        self.bb_name_index_map.get_by_right(&index).unwrap()
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_blocks(&self, index: usize) -> Vec<usize> {
        let node_index = self.bb_index_node_index_map.get_by_left(&index).unwrap();
        let from_nodes = self
            .graph
            .neighbors_directed(*node_index, Direction::Incoming);
        from_nodes
            .filter_map(|from_node_index| {
                self.bb_index_node_index_map.get_by_right(&from_node_index)
            })
            .cloned()
            .collect()
    }

    pub fn to_blocks(&self, index: usize) -> Vec<usize> {
        let node_index = self.bb_index_node_index_map.get_by_left(&index).unwrap();
        let from_nodes = self
            .graph
            .neighbors_directed(*node_index, Direction::Outgoing);
        from_nodes
            .filter_map(|from_node_index| {
                self.bb_index_node_index_map.get_by_right(&from_node_index)
            })
            .cloned()
            .collect()
    }
}
