use itertools::Itertools;
use petgraph::prelude::*;

use crate::utility::graph::kosaraju_scc_with_filter;

// todo: This is binded, maybe need an unbinded version
#[derive(Clone, Debug)]
pub struct BindedScc<'a> {
    graph: &'a DiGraph<(), (), usize>,
    pub nodes: Vec<usize>,
    pub top_level: bool,
}

impl<'a> BindedScc<'a> {
    pub fn new(
        graph: &'a DiGraph<(), (), usize>,
        nodes: impl IntoIterator<Item = usize>,
        top_level: bool,
    ) -> Self {
        Self {
            graph,
            nodes: nodes.into_iter().collect(),
            top_level,
        }
    }

    pub fn edges(&self) -> Vec<(usize, usize)> {
        self.graph
            .edge_references()
            .filter(|edge| {
                self.nodes.contains(&edge.source().index())
                    && self.nodes.contains(&edge.target().index())
            })
            .map(|it| (it.source().index(), it.target().index()))
            .collect()
    }

    pub fn entry_edges(&self) -> Vec<(usize, usize)> {
        self.graph
            .edge_references()
            .filter(|edge| {
                !self.nodes.contains(&edge.source().index())
                    && self.nodes.contains(&edge.target().index())
            })
            .map(|it| (it.source().index(), it.target().index()))
            .collect()
    }

    pub fn entry_nodes(&self) -> Vec<usize> {
        if self.top_level || self.nodes.len() == 1 {
            vec![self.nodes[0]]
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
        self.graph
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
    pub fn top_level_sccs(&self) -> Option<Vec<BindedScc<'a>>> {
        let entry_nodes = self.entry_nodes();
        if entry_nodes.len() != 1 {
            None
        } else {
            let entry_node = entry_nodes[0];
            let backedges: Vec<_> = self
                .graph
                .edges_directed(entry_node.into(), Incoming)
                .map(|it| it.id())
                .collect();
            let sccs = kosaraju_scc_with_filter(
                self.graph,
                entry_nodes[0].into(),
                |node| self.nodes.contains(&node.index()),
                |edge| !backedges.contains(&edge),
            );
            let result = sccs
                .into_iter()
                .map(|content| {
                    Self::new(self.graph, content.into_iter().map(NodeIndex::index), false)
                })
                .collect();
            Some(result)
        }
    }

    pub fn first_irreducible_sub_scc(&self) -> Option<BindedScc<'a>> {
        if self.nodes.len() == 1 {
            return None;
        } else if !self.reduciable() {
            return Some(self.clone());
        } else {
            let sccs = self.top_level_sccs().unwrap();
            for scc in sccs {
                if let Some(first_irreducible) = scc.first_irreducible_sub_scc() {
                    return Some(first_irreducible);
                }
            }
        }
        None
    }

    pub fn contains(&self, node: usize) -> bool {
        self.nodes.contains(&node)
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
        let scc = BindedScc::new(&graph, 0..10, true);
        dbg!(scc.top_level_sccs());
        dbg!(scc.first_irreducible_sub_scc());
    }
}
