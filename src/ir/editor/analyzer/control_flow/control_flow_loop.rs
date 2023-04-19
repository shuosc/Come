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
        let mut new_backedges = Vec::new();
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
}
