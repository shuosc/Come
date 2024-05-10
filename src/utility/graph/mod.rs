use std::collections::HashMap;

use petgraph::{
    algo::dominators::Dominators,
    visit::{
        EdgeRef, GraphBase, IntoEdgesDirected, IntoNeighbors, IntoNeighborsDirected,
        IntoNodeIdentifiers, VisitMap, Visitable,
    },
    Direction,
};
use std::{fmt::Debug, hash::Hash};
pub mod subgraph;
/// Initially implemented bu @m4b in [petgraph#178](https://github.com/petgraph/petgraph/pull/178).
///
/// This function will return dominance frontiers of a graph,
/// which represent join points in a control flow graph,
/// and have many applications like generating static single assignment form in a compiler.
///
/// The algorithm is mentioned in ["Simple, Fast Dominance Algorithm"][0] discovered by Cooper et al.
///
/// The algorithm is **O(|V|²)** in the worst case,
/// but in most real world cases it has almost linear complexity.
///
/// `graph` must be the same, un-mutated graph that the `dominators` was constructed from.
///
/// Panic when there are nodes unreachable from the root node `dominators` constructed with.
///
/// [0]: http://www.cs.rice.edu/~keith/EMBED/dom.pdf
pub fn dominance_frontiers<N, G>(
    dominators: &Dominators<N>,
    graph: G,
) -> HashMap<G::NodeId, Vec<G::NodeId>>
where
    N: Copy + Eq + Hash,
    <G as Visitable>::Map: VisitMap<N>,
    G: IntoNeighborsDirected
        + IntoNodeIdentifiers
        + IntoNeighbors
        + Visitable
        + GraphBase<NodeId = N>,
    <G as IntoNeighborsDirected>::NeighborsDirected: Clone,
    <G as GraphBase>::NodeId: Eq + Hash + Ord,
{
    let mut frontiers = HashMap::<G::NodeId, Vec<G::NodeId>>::from_iter(
        graph.node_identifiers().map(|v| (v, vec![])),
    );

    for node in graph.node_identifiers() {
        let (predecessors, predecessors_len) = {
            let ret = graph.neighbors_directed(node, Direction::Incoming);
            let count = ret.clone().count();
            (ret, count)
        };

        if predecessors_len >= 2 {
            for p in predecessors {
                let mut runner = p;
                if let Some(dominator) = dominators.immediate_dominator(node) {
                    while runner != dominator {
                        frontiers.entry(runner).or_insert(vec![]).push(node);
                        runner = dominators.immediate_dominator(runner).unwrap();
                    }
                }
            }
            for (_, frontier) in frontiers.iter_mut() {
                frontier.sort();
                frontier.dedup();
            }
        }
    }
    frontiers
}

fn sort_by_dfs_order<G, FN, FE>(
    current_at: G::NodeId,
    visited: &mut Vec<G::NodeId>,
    result: &mut Vec<G::NodeId>,
    g: G,
    mut node_filter: FN,
    mut edge_filter: FE,
) where
    G: IntoNeighborsDirected + IntoEdgesDirected + Visitable + IntoNodeIdentifiers,
    G::NodeId: Debug,
    FN: FnMut(G::NodeId) -> bool + Copy,
    FE: FnMut(G::EdgeId) -> bool + Copy,
{
    if visited.contains(&current_at) {
        return;
    }
    visited.push(current_at);
    for edge in g.edges_directed(current_at, Direction::Outgoing) {
        if edge_filter(edge.id()) {
            let next = edge.target();
            if node_filter(next) && !visited.contains(&next) {
                sort_by_dfs_order(next, visited, result, g, node_filter, edge_filter);
            }
        }
    }
    result.push(current_at);
}

fn collect_scc<G, FN, FE>(
    current_at: G::NodeId,
    visited: &mut Vec<G::NodeId>,
    g: G,
    mut node_filter: FN,
    mut edge_filter: FE,
) -> Vec<G::NodeId>
where
    G: IntoNeighborsDirected + IntoEdgesDirected + Visitable + IntoNodeIdentifiers,
    G::NodeId: Debug,
    FN: FnMut(G::NodeId) -> bool + Copy,
    FE: FnMut(G::EdgeId) -> bool + Copy,
{
    if visited.contains(&current_at) {
        return Vec::new();
    }
    visited.push(current_at);
    let mut result = vec![current_at];
    for edge in g.edges_directed(current_at, Direction::Incoming) {
        if edge_filter(edge.id()) {
            let next = edge.source();
            if node_filter(next) && !visited.contains(&next) {
                let inner = collect_scc(next, visited, g, node_filter, edge_filter);
                result.extend_from_slice(&inner);
            }
        }
    }
    result
}

pub fn kosaraju_scc_with_filter<G, FN, FE>(
    g: G,
    root: G::NodeId,
    node_filter: FN,
    edge_filter: FE,
) -> Vec<Vec<G::NodeId>>
where
    G: IntoNeighborsDirected + IntoEdgesDirected + Visitable + IntoNodeIdentifiers,
    G::NodeId: Debug,
    FN: FnMut(G::NodeId) -> bool + Copy,
    FE: FnMut(G::EdgeId) -> bool + Copy,
{
    let mut visited = vec![];
    let mut stack = vec![];
    sort_by_dfs_order(root, &mut visited, &mut stack, g, node_filter, edge_filter);
    visited.clear();
    let mut sccs = vec![];
    while let Some(node) = stack.pop() {
        let scc = collect_scc(node, &mut visited, g, node_filter, edge_filter);
        sccs.push(scc);
        while stack.last().map_or(false, |&x| visited.contains(&x)) {
            stack.pop();
        }
    }
    sccs
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::{algo::dominators::simple_fast, prelude::DiGraph, Graph};

    #[test]
    fn dominance_frontiers_test() {
        let mut g = Graph::<usize, ()>::new();
        let a = g.add_node(0);
        let b = g.add_node(1);
        let c = g.add_node(2);
        let d = g.add_node(3);
        let e = g.add_node(4);
        let f = g.add_node(5);

        g.add_edge(a, b, ());
        g.add_edge(b, c, ());
        g.add_edge(b, d, ());
        g.add_edge(c, e, ());
        g.add_edge(d, e, ());
        g.add_edge(e, f, ());
        g.add_edge(a, f, ());

        let dom = simple_fast(&g, a);

        assert_eq!(dom.immediate_dominator(a), None);
        assert_eq!(dom.immediate_dominator(b).unwrap(), a);
        assert_eq!(dom.immediate_dominator(c).unwrap(), b);
        assert_eq!(dom.immediate_dominator(d).unwrap(), b);
        assert_eq!(dom.immediate_dominator(e).unwrap(), b);
        assert_eq!(dom.immediate_dominator(f).unwrap(), a);

        let frontiers = dominance_frontiers(&dom, &g);

        assert_eq!(frontiers.len(), 6);
        assert_eq!(frontiers[&a], vec![]);
        assert_eq!(frontiers[&b], vec![f]);
        assert_eq!(frontiers[&c], vec![e]);
        assert_eq!(frontiers[&d], vec![e]);
        assert_eq!(frontiers[&e], vec![f]);
        assert_eq!(frontiers[&f], vec![]);
    }

    #[test]
    fn test_kosaraju_scc_with_filter() {
        let mut graph: DiGraph<_, _, usize> = DiGraph::default();
        let node_0 = graph.add_node(0);
        let node_1 = graph.add_node(1);
        let node_2 = graph.add_node(2);
        let node_3 = graph.add_node(3);
        let node_4 = graph.add_node(4);
        graph.add_edge(node_0, node_3, ());
        graph.add_edge(node_3, node_4, ());
        graph.add_edge(node_4, node_0, ());
        graph.add_edge(node_0, node_2, ());
        graph.add_edge(node_2, node_1, ());
        graph.add_edge(node_1, node_0, ());
        let result = kosaraju_scc_with_filter(&graph, node_0, |_| true, |_| true);
        assert_eq!(result.len(), 1);

        let mut graph: DiGraph<_, _, usize> = DiGraph::default();
        let node_0 = graph.add_node(0);
        let node_1 = graph.add_node(1);
        let node_2 = graph.add_node(2);
        let node_3 = graph.add_node(3);
        let node_4 = graph.add_node(4);
        let node_5 = graph.add_node(5);
        let node_6 = graph.add_node(6);
        let node_7 = graph.add_node(7);
        graph.add_edge(node_0, node_1, ());
        graph.add_edge(node_1, node_2, ());
        graph.add_edge(node_2, node_0, ());
        graph.add_edge(node_2, node_3, ());
        graph.add_edge(node_3, node_4, ());
        graph.add_edge(node_4, node_5, ());
        graph.add_edge(node_4, node_7, ());
        graph.add_edge(node_5, node_6, ());
        graph.add_edge(node_6, node_4, ());
        graph.add_edge(node_6, node_7, ());
        let result = kosaraju_scc_with_filter(&graph, node_0, |_| true, |_| true);
        assert_eq!(result.len(), 4);
        let node_0_in_scc = result.iter().find(|scc| scc.contains(&node_0)).unwrap();
        assert_eq!(node_0_in_scc.len(), 3);
        let node_3_in_scc = result.iter().find(|scc| scc.contains(&node_3)).unwrap();
        assert_eq!(node_3_in_scc.len(), 1);
        let node_4_in_scc = result.iter().find(|scc| scc.contains(&node_4)).unwrap();
        assert_eq!(node_4_in_scc.len(), 3);
        let node_7_in_scc = result.iter().find(|scc| scc.contains(&node_7)).unwrap();
        assert_eq!(node_7_in_scc.len(), 1);

        let mut graph: DiGraph<_, _, usize> = DiGraph::default();
        let node_0 = graph.add_node(0);
        let node_1 = graph.add_node(1);
        let node_2 = graph.add_node(2);
        let node_3 = graph.add_node(3);
        let node_4 = graph.add_node(4);
        let node_5 = graph.add_node(5);
        let node_6 = graph.add_node(6);
        graph.add_edge(node_0, node_1, ());
        graph.add_edge(node_0, node_2, ());
        graph.add_edge(node_1, node_2, ());
        graph.add_edge(node_2, node_3, ());
        graph.add_edge(node_3, node_4, ());
        graph.add_edge(node_4, node_1, ());
        graph.add_edge(node_3, node_1, ());
        graph.add_edge(node_2, node_6, ());
        graph.add_edge(node_4, node_6, ());
        graph.add_edge(node_6, node_5, ());
        graph.add_edge(node_5, node_4, ());
        let result = kosaraju_scc_with_filter(&graph, node_0, |_| true, |_| true);
        assert_eq!(result.len(), 2);
        let node_0_in_scc = result.iter().find(|scc| scc.contains(&node_0)).unwrap();
        assert_eq!(node_0_in_scc.len(), 1);
        let node_1_in_scc = result.iter().find(|scc| scc.contains(&node_1)).unwrap();
        assert_eq!(node_1_in_scc.len(), 6);

        let mut graph: DiGraph<_, _, usize> = DiGraph::default();
        let node_0 = graph.add_node(0);
        let node_1 = graph.add_node(1);
        let node_2 = graph.add_node(2);
        let node_3 = graph.add_node(3);
        let node_4 = graph.add_node(4);
        let node_5 = graph.add_node(5);
        let node_6 = graph.add_node(6);
        let node_7 = graph.add_node(7);
        graph.add_edge(node_0, node_1, ());
        graph.add_edge(node_1, node_2, ());
        graph.add_edge(node_2, node_3, ());
        graph.add_edge(node_3, node_4, ());
        graph.add_edge(node_4, node_5, ());
        graph.add_edge(node_5, node_0, ());
        graph.add_edge(node_6, node_1, ());
        graph.add_edge(node_2, node_7, ());
        graph.add_edge(node_7, node_6, ());
        let result = kosaraju_scc_with_filter(&graph, node_0, |_| true, |_| true);
        assert_eq!(result.len(), 1);
        let result = kosaraju_scc_with_filter(
            &graph,
            node_0,
            |_| true,
            |edge| {
                let (from, to) = graph.edge_endpoints(edge).unwrap();
                !(from.index() == 5 && to.index() == 0)
            },
        );
        assert_eq!(result.len(), 5);
        let node_2_in_scc = result.iter().find(|scc| scc.contains(&node_2)).unwrap();
        assert_eq!(node_2_in_scc.len(), 4);
        let node_2_in_scc = result.iter().find(|scc| scc.contains(&node_2)).unwrap();
        assert_eq!(node_2_in_scc.len(), 4);

        let mut graph: DiGraph<_, _, usize> = DiGraph::default();
        let node_0 = graph.add_node(0);
        let node_1 = graph.add_node(1);
        let node_2 = graph.add_node(2);
        let node_3 = graph.add_node(3);
        let node_4 = graph.add_node(4);
        let node_5 = graph.add_node(5);
        let node_6 = graph.add_node(6);
        let node_7 = graph.add_node(7);
        let node_8 = graph.add_node(8);
        let node_9 = graph.add_node(9);
        let node_10 = graph.add_node(10);
        graph.add_edge(node_0, node_1, ());
        graph.add_edge(node_1, node_2, ());
        graph.add_edge(node_1, node_6, ());
        graph.add_edge(node_2, node_3, ());
        graph.add_edge(node_3, node_5, ());
        graph.add_edge(node_4, node_2, ());
        graph.add_edge(node_4, node_9, ());
        graph.add_edge(node_5, node_4, ());
        graph.add_edge(node_5, node_10, ());
        graph.add_edge(node_6, node_4, ());
        graph.add_edge(node_6, node_7, ());
        graph.add_edge(node_7, node_8, ());
        graph.add_edge(node_8, node_3, ());
        graph.add_edge(node_8, node_9, ());
        graph.add_edge(node_9, node_7, ());
        let result = kosaraju_scc_with_filter(&graph, node_0, |_| true, |_| true);
        assert_eq!(result.len(), 5);
        let node_2_in_scc = result.iter().find(|scc| scc.contains(&node_2)).unwrap();
        assert_eq!(node_2_in_scc.len(), 7);
        let node_0_in_scc = result.iter().find(|scc| scc.contains(&node_0)).unwrap();
        assert_eq!(node_0_in_scc.len(), 1);
    }
}
