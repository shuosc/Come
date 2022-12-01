use std::collections::HashMap;

use petgraph::{
    algo::dominators::Dominators,
    visit::{
        GraphBase, IntoNeighbors, IntoNeighborsDirected, IntoNodeIdentifiers, VisitMap, Visitable,
    },
    Direction,
};
use std::hash::Hash;

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
/// [0]: http://www.cs.rice.edu/~keith/EMBED/dom.pdf
pub fn dominance_frontiers<N, G>(
    dorminators: &Dominators<N>,
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
                if let Some(dominator) = dorminators.immediate_dominator(node) {
                    while runner != dominator {
                        frontiers.entry(runner).or_insert(vec![]).push(node);
                        runner = dorminators.immediate_dominator(runner).unwrap();
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

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::{algo::dominators::simple_fast, Graph};

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
}
