use std::fmt;

use crate::utility::graph::{kosaraju_scc_with_filter, subgraph::SubGraph};
use delegate::delegate;

use itertools::Itertools;
use petgraph::{
    algo::all_simple_paths,
    graph,
    prelude::*,
    visit::{IntoNeighborsDirected, NodeFiltered, NodeRef},
};

#[derive(Clone)]
pub struct Scc {
    pub nodes: Vec<usize>,
    pub top_level: bool,
}

impl Scc {
    pub fn new(nodes: impl IntoIterator<Item = usize>, top_level: bool) -> Self {
        Self {
            nodes: nodes.into_iter().collect(),
            top_level,
        }
    }

    pub fn is_trivial(&self) -> bool {
        self.nodes.len() == 1
    }

    pub fn edges(&self, graph: &DiGraph<(), (), usize>) -> Vec<(usize, usize)> {
        graph
            .edge_references()
            .filter(|edge| {
                self.nodes.contains(&edge.source().index())
                    && self.nodes.contains(&edge.target().index())
            })
            .map(|it| (it.source().index(), it.target().index()))
            .collect()
    }

    pub fn entry_edges(&self, graph: &DiGraph<(), (), usize>) -> Vec<(usize, usize)> {
        graph
            .edge_references()
            .filter(|edge| {
                !self.nodes.contains(&edge.source().index())
                    && self.nodes.contains(&edge.target().index())
            })
            .map(|it| (it.source().index(), it.target().index()))
            .collect()
    }

    pub fn entry_nodes(&self, graph: &DiGraph<(), (), usize>) -> Vec<usize> {
        if self.top_level || self.nodes.len() == 1 {
            vec![self.nodes[0]]
        } else {
            self.entry_edges(graph)
                .into_iter()
                .map(|(_, to)| to)
                .sorted()
                .dedup()
                .collect()
        }
    }

    pub fn edges_into_entry_nodes(&self, graph: &DiGraph<(), (), usize>) -> Vec<(usize, usize)> {
        graph
            .edge_references()
            .filter(|edge| self.entry_nodes(graph).contains(&edge.target().index()))
            .map(|it| (it.source().index(), it.target().index()))
            .collect()
    }

    pub fn reduciable(&self, graph: &DiGraph<(), (), usize>) -> bool {
        self.entry_nodes(graph).len() == 1
    }

    /// Returns all top level sccs for a reducible subgraph.
    /// Return None if the subgraph is not reducible.
    pub fn top_level_sccs(&self, graph: &DiGraph<(), (), usize>) -> Option<Vec<Scc>> {
        let entry_nodes = self.entry_nodes(graph);
        if entry_nodes.len() != 1 {
            None
        } else {
            let entry_node = entry_nodes[0];
            let node_filtered =
                NodeFiltered::from_fn(graph, |node| self.nodes.contains(&node.index()));
            let subgraph = SubGraph(node_filtered);
            let largest_simple_loop = subgraph
                .neighbors_directed(entry_node.into(), Incoming)
                .flat_map(|pred| {
                    all_simple_paths::<Vec<_>, _>(&subgraph, entry_node.into(), pred, 1, None)
                        .max_by(|a, b| a.len().cmp(&b.len()))
                })
                .max_by(|a, b| a.len().cmp(&b.len()));
            dbg!(&largest_simple_loop);
            let backedge = if let Some(mut largest_simple_loops) = largest_simple_loop {
                let last_node = largest_simple_loops.pop().unwrap();
                graph.find_edge(last_node, entry_node.into())
            } else {
                None
            };
            let backedge_info = backedge.and_then(|e| graph.edge_endpoints(e));
            println!("entry: {:?}, {:?}", entry_node, backedge_info);
            let sccs = kosaraju_scc_with_filter(
                graph,
                entry_nodes[0].into(),
                |node| self.nodes.contains(&node.index()),
                |edge| Some(edge) != backedge,
            );
            let result = sccs
                .into_iter()
                .map(|content| Self::new(content.into_iter().map(NodeIndex::index), false))
                .collect();
            Some(result)
        }
    }

    pub fn first_irreducible_sub_scc(&self, graph: &DiGraph<(), (), usize>) -> Option<Scc> {
        if self.nodes.len() == 1 {
            return None;
        } else if !self.reduciable(graph) {
            return Some(self.clone());
        } else {
            let sccs = self.top_level_sccs(graph).unwrap();
            for scc in sccs {
                if let Some(first_irreducible) = scc.first_irreducible_sub_scc(graph) {
                    return Some(first_irreducible);
                }
            }
        }
        None
    }

    pub fn contains(&self, node: usize) -> bool {
        self.nodes.contains(&node)
    }

    /// Returns the smallest non trivial (ie. not a single node) scc
    /// the node is in.
    pub fn smallest_non_trivial_scc_node_in(
        &self,
        graph: &DiGraph<(), (), usize>,
        node: usize,
    ) -> Option<Scc> {
        if !self.contains(node) {
            None
        } else if self.is_trivial() {
            None
        } else if let Some(sub_sccs) = self.top_level_sccs(graph) {
            for sub_scc in sub_sccs {
                if sub_scc.is_trivial() && sub_scc.contains(node) {
                    return Some(self.clone());
                } else if let Some(result) = sub_scc.smallest_non_trivial_scc_node_in(graph, node) {
                    return Some(result);
                }
            }
            unreachable!()
        } else {
            debug_assert!(!self.reduciable(graph));
            Some(self.clone())
        }
    }
}

impl fmt::Display for Scc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.nodes)
    }
}

impl fmt::Debug for Scc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.nodes)
    }
}

#[derive(Clone)]
pub struct BindedScc<'bind> {
    graph: &'bind DiGraph<(), (), usize>,
    item: Scc,
}

impl fmt::Display for BindedScc<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.item.fmt(f)
    }
}

impl fmt::Debug for BindedScc<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.item.fmt(f)
    }
}

impl<'bind> BindedScc<'bind> {
    pub fn new(graph: &'bind DiGraph<(), (), usize>, item: Scc) -> Self {
        Self { graph, item }
    }
    delegate! {
        to self.item {
            pub fn is_trivial(&self) -> bool;
            pub fn edges(&self, [self.graph]) -> Vec<(usize, usize)>;
            pub fn entry_edges(&self, [self.graph]) -> Vec<(usize, usize)>;
            pub fn entry_nodes(&self, [self.graph]) -> Vec<usize>;
            pub fn edges_into_entry_nodes(&self, [self.graph]) -> Vec<(usize, usize)>;
            pub fn reduciable(&self, [self.graph]) -> bool;
            pub fn contains(&self, node: usize) -> bool;
        }
    }
    pub fn top_level_sccs(&self) -> Option<Vec<Self>> {
        self.item
            .top_level_sccs(self.graph)
            .map(|it| it.into_iter().map(|it| it.bind(self.graph)).collect_vec())
    }
    pub fn first_irreducible_sub_scc(&self) -> Option<Self> {
        self.item
            .first_irreducible_sub_scc(self.graph)
            .map(|it| it.bind(self.graph))
    }
    pub fn smallest_non_trivial_scc_node_in(&self, node: usize) -> Option<Self> {
        self.item
            .smallest_non_trivial_scc_node_in(self.graph, node)
            .map(|it| it.bind(self.graph))
    }
    pub fn top_level(&self) -> bool {
        self.item.top_level
    }
}

impl<'item, 'bind: 'item> Scc {
    pub fn bind(&'item self, graph: &'bind DiGraph<(), (), usize>) -> BindedScc<'bind> {
        BindedScc::new(graph, self.clone())
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
        let scc = Scc::new(0..10, true);
        let scc = BindedScc::new(&graph, scc);
        println!("{:?}", scc.top_level_sccs());
        println!("{:?}", scc.first_irreducible_sub_scc());
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
        let scc = Scc::new(0..6, true);
        let scc = BindedScc::new(&graph, scc);
        let top_level_sccs = scc.top_level_sccs().unwrap();
        println!("{:?}", &top_level_sccs);
        let contains_recursive = &top_level_sccs[1];
        println!("{:?}", contains_recursive.top_level_sccs());
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
        let node_7 = graph.add_node(());
        graph.add_edge(node_0, node_1, ());
        graph.add_edge(node_1, node_2, ());
        graph.add_edge(node_2, node_3, ());
        graph.add_edge(node_3, node_1, ());
        graph.add_edge(node_1, node_4, ());
        graph.add_edge(node_4, node_5, ());
        graph.add_edge(node_5, node_1, ());
        graph.add_edge(node_3, node_6, ());
        graph.add_edge(node_2, node_4, ());
        graph.add_edge(node_4, node_7, ());
        let scc = Scc::new(0..8, true);
        let scc = BindedScc::new(&graph, scc);
        let top_level_sccs = scc.top_level_sccs().unwrap();
        println!("{:?}", top_level_sccs);
        let contains_recursive = &top_level_sccs[1];
        println!("{:?}", contains_recursive.top_level_sccs());
    }
}
