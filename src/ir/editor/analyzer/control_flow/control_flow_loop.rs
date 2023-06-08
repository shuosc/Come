use itertools::Itertools;
use petgraph::prelude::*;

use crate::utility::graph::kosaraju_scc_with_filter;

#[derive(Debug, PartialEq)]
pub enum LoopContent {
    SubLoop(Box<Loop>),
    Node(usize),
}

impl LoopContent {
    pub fn is_node_in(&self, node: NodeIndex<usize>) -> bool {
        match self {
            LoopContent::SubLoop(it) => it.is_node_in(node),
            LoopContent::Node(it) => node.index() == *it,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Loop {
    pub entries: Vec<usize>,
    pub content: Vec<LoopContent>,
}

impl Loop {
    pub fn new(
        graph: &DiGraph<(), (), usize>,
        nodes: &[NodeIndex<usize>],
        backedges: &[EdgeIndex<usize>],
    ) -> Self {
        let entries: Vec<_> = nodes
            .iter()
            .filter(|&&node| {
                graph
                    .neighbors_directed(node, Incoming)
                    .any(|from| !nodes.contains(&from))
            })
            .cloned()
            .collect();
        let mut new_backedges: Vec<EdgeIndex<usize>> = Vec::new();
        for &entry in entries.iter() {
            new_backedges.extend(
                graph
                    .edges_directed(entry, Incoming)
                    .filter(|edge| nodes.contains(&edge.source()))
                    .map(|it| it.id()),
            );
        }
        new_backedges.extend_from_slice(backedges);
        let sccs = kosaraju_scc_with_filter(
            graph,
            *entries.first().or(nodes.first()).unwrap(),
            |it| nodes.contains(&it),
            |edge| !new_backedges.contains(&edge),
        );
        if sccs.len() == 1 {
            return Self {
                entries: entries.into_iter().map(|it| it.index()).collect(),
                content: sccs[0]
                    .iter()
                    .map(|it| LoopContent::Node(it.index()))
                    .collect(),
            };
        }
        let mut content: Vec<_> = sccs
            .into_iter()
            .map(|mut scc| {
                if scc.len() == 1 {
                    LoopContent::Node(scc.pop().unwrap().index())
                } else {
                    let sub_loop = Loop::new(graph, &scc, &new_backedges);
                    LoopContent::SubLoop(Box::new(sub_loop))
                }
            })
            .collect();
        for entry in &entries {
            if !content.iter().any(|it| it.is_node_in(*entry)) {
                content.push(LoopContent::Node(entry.index()));
            }
        }
        Self {
            entries: entries.into_iter().map(|it| it.index()).collect(),
            content,
        }
    }

    pub fn first_irreducible_loop(&self) -> Option<&Loop> {
        if self.entries.len() > 1 {
            Some(self)
        } else {
            self.content
                .iter()
                .filter_map(|it| match it {
                    LoopContent::SubLoop(sub_loop) => Some(sub_loop),
                    LoopContent::Node(_) => None,
                })
                .find_map(|it| it.first_irreducible_loop())
        }
    }

    pub fn entry_info(
        &self,
        graph: &DiGraph<(), (), usize>,
    ) -> Vec<(NodeIndex<usize>, Vec<NodeIndex<usize>>)> {
        let mut result: Vec<_> = self
            .entries
            .iter()
            .map(|&entry| {
                let mut from = graph
                    .edges_directed(entry.into(), Direction::Incoming)
                    .map(|it| it.source())
                    .collect_vec();
                from.sort_unstable();
                (entry.into(), from)
            })
            .collect();
        result.sort_unstable_by_key(|it| it.0);
        result
    }

    pub fn name(&self) -> String {
        format!(
            "_loop_{}",
            self.entries.iter().map(ToString::to_string).join("_")
        )
    }

    pub fn smallest_loop_node_in(&self, node: NodeIndex<usize>) -> Option<&Loop> {
        let found_node = self
            .content
            .iter()
            .filter_map(|it| {
                if let LoopContent::Node(node) = it {
                    Some(*node)
                } else {
                    None
                }
            })
            .find(|&it| it == node.index());
        if found_node.is_some() {
            return Some(self);
        }
        self.content
            .iter()
            .filter_map(|it| match it {
                LoopContent::SubLoop(sub_loop) => Some(sub_loop),
                LoopContent::Node(_) => None,
            })
            .find_map(|it| it.smallest_loop_node_in(node))
    }

    pub fn is_node_in(&self, node: NodeIndex<usize>) -> bool {
        self.content.iter().any(|it| match it {
            LoopContent::SubLoop(sub_loop) => sub_loop.is_node_in(node),
            LoopContent::Node(n) => node.index() == *n,
        })
    }

    pub fn node_count(&self) -> usize {
        self.content
            .iter()
            .map(|it| match it {
                LoopContent::SubLoop(sub_loop) => sub_loop.node_count(),
                LoopContent::Node(_) => 1,
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_new_loop() {
        let mut graph: DiGraph<_, _, usize> = DiGraph::default();
        let node_0 = graph.add_node(());
        let node_1 = graph.add_node(());
        let node_2 = graph.add_node(());
        let node_3 = graph.add_node(());
        let node_4 = graph.add_node(());
        let node_5 = graph.add_node(());
        let node_6 = graph.add_node(());
        let node_7 = graph.add_node(());
        graph.add_edge(node_0, node_7, ());
        graph.add_edge(node_7, node_1, ());
        graph.add_edge(node_1, node_7, ());
        graph.add_edge(node_7, node_2, ());
        graph.add_edge(node_2, node_3, ());
        graph.add_edge(node_2, node_6, ());
        graph.add_edge(node_3, node_4, ());
        graph.add_edge(node_4, node_6, ());
        graph.add_edge(node_6, node_5, ());
        graph.add_edge(node_5, node_4, ());
        graph.add_edge(node_3, node_7, ());
        graph.add_edge(node_4, node_7, ());
        let result = Loop::new(
            &graph,
            &[
                node_0, node_1, node_2, node_3, node_4, node_5, node_6, node_7,
            ],
            &[],
        );
        assert_eq!(result.content.len(), 2);
        let inner_loop = result
            .content
            .iter()
            .find_map(|it| match it {
                LoopContent::SubLoop(sub_loop) => Some(sub_loop),
                LoopContent::Node(_) => None,
            })
            .unwrap()
            .as_ref();
        assert_eq!(inner_loop.entries.len(), 1);
        assert_eq!(inner_loop.entries[0], 7);
        assert_eq!(inner_loop.content.len(), 5);
    }
}
