use itertools::Itertools;
use petgraph::prelude::*;

use crate::utility::graph::kosaraju_scc_with_filter;

#[derive(Debug, PartialEq)]
pub enum LoopContent {
    SubLoop(Box<Loop>),
    Node(usize),
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
        let sccs = kosaraju_scc_with_filter(
            graph,
            |it| nodes.contains(&it),
            |edge| !backedges.contains(&edge),
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
        let mut new_backedges: Vec<EdgeIndex<usize>> = Vec::new();
        for &entry in entries.iter() {
            new_backedges.extend(
                graph
                    .edges_directed(entry, Incoming)
                    .filter(|edge| nodes.contains(&edge.source()))
                    .map(|it| it.id()),
            );
        }
        let content = sccs
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
}
