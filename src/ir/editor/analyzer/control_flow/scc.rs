use std::fmt;

use crate::utility::graph::{
    kosaraju_scc_with_filter,
    subgraph::{CFGraph, CFSubGraph},
};

use itertools::Itertools;
use petgraph::{
    algo::all_simple_paths,
    prelude::*,
    visit::{GraphBase, IntoEdgeReferences, IntoNeighborsDirected},
};

#[derive(Clone)]
pub struct BindedScc<'a> {
    pub graph_part: CFSubGraph<'a>,
    pub top_level: bool,
}

impl<'a> PartialEq for BindedScc<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.graph_part.nodes.eq(&other.graph_part.nodes)
    }
}
impl<'a> BindedScc<'a> {
    pub fn new(
        graph: &'a CFGraph,
        nodes: impl IntoIterator<Item = <CFGraph as GraphBase>::NodeId>,
        edges: impl IntoIterator<Item = <CFGraph as GraphBase>::EdgeId>,
        top_level: bool,
    ) -> Self {
        Self {
            graph_part: CFSubGraph::new(graph, nodes, edges),
            top_level,
        }
    }
    pub fn new_top_level_from_graph(graph: &'a CFGraph) -> Self {
        let nodes = graph.node_indices();
        let edges = graph.edge_indices();
        Self::new(graph, nodes, edges, true)
    }
    pub fn is_trivial(&self) -> bool {
        self.graph_part.nodes.len() == 1
    }
    pub fn contains(&self, node: usize) -> bool {
        self.graph_part.nodes.contains(&node.into())
    }
    pub fn edges(&self) -> Vec<(usize, usize)> {
        self.graph_part
            .edge_references()
            .filter(|edge| {
                self.graph_part.nodes.contains(&edge.source())
                    && self.graph_part.nodes.contains(&edge.target())
            })
            .map(|it| (it.source().index(), it.target().index()))
            .collect()
    }
    pub fn entry_edges(&self) -> Vec<(usize, usize)> {
        self.graph_part
            .graph
            .edge_references()
            .filter(|edge| {
                !self.graph_part.nodes.contains(&edge.source())
                    && self.graph_part.nodes.contains(&edge.target())
            })
            .map(|it| (it.source().index(), it.target().index()))
            .collect()
    }

    pub fn entry_nodes(&self) -> Vec<usize> {
        if self.top_level || self.graph_part.nodes.len() == 1 {
            vec![self.graph_part.nodes[0].index()]
        } else {
            self.entry_edges()
                .into_iter()
                .map(|(_, to)| to)
                .sorted()
                .dedup()
                .collect()
        }
    }

    pub fn edges_into_entry_nodes(&self) -> Vec<(usize, usize)> {
        self.graph_part
            .edge_references()
            .filter(|edge| self.entry_nodes().contains(&edge.target().index()))
            .map(|it| (it.source().index(), it.target().index()))
            .collect()
    }

    pub fn extern_edges_into_entry_nodes(&self) -> Vec<(usize, usize)> {
        self.graph_part
            .graph
            .edge_references()
            .filter(|edge| self.entry_nodes().contains(&edge.target().index()))
            .map(|it| (it.source().index(), it.target().index()))
            .collect()
    }

    pub fn reduciable(&self) -> bool {
        self.entry_nodes().len() == 1
    }

    /// Returns all top level sccs for a reducible subgraph.
    /// Return None if the subgraph is not reducible.
    pub fn top_level_sccs(&self) -> Option<Vec<Self>> {
        let entry_nodes = self.entry_nodes();
        if entry_nodes.len() != 1 {
            None
        } else {
            let entry_node = entry_nodes[0];
            let largest_simple_loop = self
                .graph_part
                .neighbors_directed(entry_node.into(), Incoming)
                .flat_map(|pred| {
                    all_simple_paths::<Vec<_>, _>(
                        &self.graph_part,
                        entry_node.into(),
                        pred,
                        0,
                        None,
                    )
                    .max_by(|a, b| a.len().cmp(&b.len()))
                })
                .max_by(|a, b| a.len().cmp(&b.len()));
            let backedge = if let Some(mut largest_simple_loops) = largest_simple_loop {
                let last_node = largest_simple_loops.pop().unwrap();
                self.graph_part.find_edge(last_node, entry_node.into())
            } else {
                None
            };
            let edges_without_backedge = if let Some(backedge) = backedge {
                self.graph_part
                    .edges
                    .iter()
                    .filter(|&&e| e != backedge)
                    .cloned()
                    .collect()
            } else {
                self.graph_part.edges.clone()
            };
            let sccs = kosaraju_scc_with_filter(
                &self.graph_part,
                entry_nodes[0].into(),
                |_| true,
                |e| Some(e) != backedge,
            );
            let result = sccs
                .into_iter()
                .map(|content| {
                    Self::new(
                        self.graph_part.graph,
                        content,
                        edges_without_backedge.clone(),
                        false,
                    )
                })
                .collect();
            Some(result)
        }
    }

    pub fn first_irreducible_sub_scc(&self) -> Option<Self> {
        if self.graph_part.nodes.len() == 1 {
            return None;
        } else if !self.reduciable() {
            return Some(self.clone());
        } else {
            // dbg!(self.top_level_sccs());
            // if self
            //     .top_level_sccs()
            //     .map(|it| it.len() != 2)
            //     .unwrap_or(false)
            // {
            //     return None;
            // }
            let sccs = self.top_level_sccs().unwrap();
            for scc in sccs {
                // if &scc == self {
                //     return Some(scc);
                // }
                if let Some(first_irreducible) = scc.first_irreducible_sub_scc() {
                    return Some(first_irreducible);
                }
            }
        }
        None
    }

    /// Returns the smallest non trivial (ie. not a single node) scc
    /// the node is in.
    pub fn smallest_non_trivial_scc_node_in(&self, node: usize) -> Option<Self> {
        if !self.contains(node) || self.is_trivial() {
            None
        } else if let Some(sub_sccs) = self.top_level_sccs() {
            for sub_scc in sub_sccs {
                if sub_scc.is_trivial() && sub_scc.contains(node) {
                    return Some(self.clone());
                } else if let Some(result) = sub_scc.smallest_non_trivial_scc_node_in(node) {
                    return Some(result);
                }
            }
            unreachable!()
        } else {
            debug_assert!(!self.reduciable());
            Some(self.clone())
        }
    }
}

impl<'a> fmt::Display for BindedScc<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Scc{{{:?}, top_level: {}}}",
            self.graph_part.nodes, self.top_level
        )
    }
}

impl<'a> fmt::Debug for BindedScc<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Scc{{{:?}, top_level: {}}}",
            self.graph_part.nodes, self.top_level
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_level_scc() {
        let mut graph: DiGraph<_, _, usize> = DiGraph::default();
        let node_0 = graph.add_node(());
        let node_1 = graph.add_node(());
        let node_2 = graph.add_node(());
        let node_3 = graph.add_node(());
        let node_4 = graph.add_node(());
        let node_5 = graph.add_node(());
        let node_6 = graph.add_node(());
        let node_7 = graph.add_node(());
        let node_8 = graph.add_node(());
        let node_9 = graph.add_node(());
        let node_10 = graph.add_node(());
        graph.add_edge(node_0, node_1, ());
        graph.add_edge(node_1, node_2, ());
        graph.add_edge(node_2, node_3, ());
        graph.add_edge(node_2, node_7, ());
        graph.add_edge(node_3, node_4, ());
        graph.add_edge(node_3, node_8, ());
        graph.add_edge(node_7, node_5, ());
        graph.add_edge(node_7, node_9, ());
        graph.add_edge(node_9, node_8, ());
        graph.add_edge(node_8, node_9, ());
        graph.add_edge(node_5, node_2, ());
        graph.add_edge(node_4, node_6, ());
        graph.add_edge(node_6, node_2, ());
        graph.add_edge(node_6, node_10, ());
        graph.add_edge(node_10, node_6, ());
        graph.add_edge(node_10, node_4, ());

        let scc = BindedScc::new_top_level_from_graph(&graph);
        println!("{:?}", scc.top_level_sccs());
        println!(
            "first_irreducible_sub_scc={:?}",
            scc.first_irreducible_sub_scc()
        );
    }

    #[test]
    fn test_top_level_scc_recursive() {
        let mut graph: DiGraph<_, _, usize> = DiGraph::default();
        let node_0 = graph.add_node(());
        let node_1 = graph.add_node(());
        let node_2 = graph.add_node(());
        let node_3 = graph.add_node(());
        let node_4 = graph.add_node(());
        let node_5 = graph.add_node(());
        graph.add_edge(node_0, node_1, ());
        graph.add_edge(node_1, node_2, ());
        graph.add_edge(node_2, node_3, ());
        graph.add_edge(node_3, node_4, ());
        graph.add_edge(node_4, node_1, ());
        graph.add_edge(node_3, node_1, ());
        graph.add_edge(node_4, node_5, ());
        let scc = BindedScc::new_top_level_from_graph(&graph);
        let top_level_sccs = scc.top_level_sccs().unwrap();
        println!("{:?}", &top_level_sccs);
        let scc = &top_level_sccs[1];
        println!("{:?}", &scc);
        let top_level_sccs = scc.top_level_sccs().unwrap();
        println!("{:?}", &top_level_sccs);
        let scc = &top_level_sccs[0];
        let top_level_sccs = scc.top_level_sccs().unwrap();
        println!("{:?}", &top_level_sccs);
    }

    #[test]
    fn test_top_level_scc_recursive2() {
        let mut graph: DiGraph<_, _, usize> = DiGraph::default();
        let node_0 = graph.add_node(());
        let node_1 = graph.add_node(());
        let node_2 = graph.add_node(());
        let node_3 = graph.add_node(());
        let node_4 = graph.add_node(());
        let node_5 = graph.add_node(());
        let node_6 = graph.add_node(());
        graph.add_edge(node_0, node_1, ());
        graph.add_edge(node_1, node_2, ());
        graph.add_edge(node_2, node_3, ());
        graph.add_edge(node_3, node_4, ());
        graph.add_edge(node_4, node_5, ());
        graph.add_edge(node_3, node_6, ());
        graph.add_edge(node_6, node_1, ());
        graph.add_edge(node_4, node_6, ());

        let scc = BindedScc::new_top_level_from_graph(&graph);
        let top_level_sccs = scc.top_level_sccs().unwrap();
        println!("{:?}", &top_level_sccs);
        let scc = &top_level_sccs[1];
        let top_level_sccs = scc.top_level_sccs().unwrap();
        println!("{:?}", &top_level_sccs);
        // let scc = &top_level_sccs[0];
        // let top_level_sccs = scc.top_level_sccs().unwrap();
        // println!("{:?}", &top_level_sccs);
    }
    #[test]
    fn test_top_level_scc_recursive3() {
        let mut graph: DiGraph<_, _, usize> = DiGraph::default();
        let node_0 = graph.add_node(());
        let node_1 = graph.add_node(());
        let node_2 = graph.add_node(());
        let node_3 = graph.add_node(());
        let node_4 = graph.add_node(());
        let node_5 = graph.add_node(());
        let node_6 = graph.add_node(());
        graph.add_edge(node_0, node_1, ());
        graph.add_edge(node_1, node_2, ());
        graph.add_edge(node_2, node_3, ());
        graph.add_edge(node_3, node_1, ());
        graph.add_edge(node_1, node_4, ());
        graph.add_edge(node_4, node_5, ());
        graph.add_edge(node_5, node_1, ());
        graph.add_edge(node_3, node_6, ());

        let scc = BindedScc::new_top_level_from_graph(&graph);
        let top_level_sccs = scc.top_level_sccs().unwrap();
        println!("{:?}", &top_level_sccs);
        let scc = &top_level_sccs[1];
        let top_level_sccs = scc.top_level_sccs().unwrap();
        println!("{:?}", &top_level_sccs);
        // let scc = &top_level_sccs[0];
        // let top_level_sccs = scc.top_level_sccs().unwrap();
        // println!("{:?}", &top_level_sccs);
    }

    #[test]
    fn test_top_level_strange() {
        let mut graph: DiGraph<_, _, usize> = DiGraph::default();
        let node_0 = graph.add_node(());
        let node_1 = graph.add_node(());
        let node_2 = graph.add_node(());
        graph.add_edge(node_0, node_1, ());
        graph.add_edge(node_1, node_2, ());
        graph.add_edge(node_2, node_0, ());
        graph.add_edge(node_0, node_2, ());
        graph.add_edge(node_2, node_1, ());
        graph.add_edge(node_1, node_0, ());

        let scc = BindedScc::new_top_level_from_graph(&graph);
        println!("{:?}", scc.top_level_sccs());
        println!(
            "first_irreducible_sub_scc={:?}",
            scc.first_irreducible_sub_scc()
        );
    }

    #[test]
    fn test_top_level_strange_0() {
        let mut graph: DiGraph<_, _, usize> = DiGraph::default();
        let node_0 = graph.add_node(());
        let node_1 = graph.add_node(());
        let node_2 = graph.add_node(());
        let node_3 = graph.add_node(());
        graph.add_edge(node_0, node_1, ());
        graph.add_edge(node_1, node_2, ());
        graph.add_edge(node_1, node_3, ());
        graph.add_edge(node_2, node_3, ());
        graph.add_edge(node_3, node_1, ());
        graph.add_edge(node_2, node_1, ());
        graph.add_edge(node_3, node_2, ());

        let scc = BindedScc::new_top_level_from_graph(&graph);

        println!(
            "first_irreducible_sub_scc={:?}",
            scc.first_irreducible_sub_scc()
        );
    }
}
