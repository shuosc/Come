use std::fmt::{self, Debug};

use itertools::Itertools;
use petgraph::{
    graph::DiGraph,
    visit::{
        FilterNode, GraphBase, GraphRef, IntoNeighbors, IntoNeighborsDirected, IntoNodeIdentifiers,
        IntoNodeReferences, NodeCount, NodeFiltered,
    },
    Direction,
};

type CFGraph = DiGraph<(), (), usize>;

type FilteredCFGraph<'a, F> = NodeFiltered<&'a CFGraph, F>;

#[derive(Copy, Clone)]
pub struct SubGraph<'a, F: FilterNode<<CFGraph as GraphBase>::NodeId>>(pub FilteredCFGraph<'a, F>);

impl<F: FilterNode<<CFGraph as GraphBase>::NodeId>> fmt::Debug for SubGraph<'_, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nodes = self
            .0
            .node_references()
            .filter(|(it, _)| self.0 .1.include_node(*it))
            .collect_vec();
        f.debug_tuple("SubGraph").field(&nodes).finish()
    }
}

impl<'a, F: FilterNode<<CFGraph as GraphBase>::NodeId>> GraphBase for SubGraph<'a, F> {
    type EdgeId = <FilteredCFGraph<'a, F> as GraphBase>::EdgeId;

    type NodeId = <FilteredCFGraph<'a, F> as GraphBase>::NodeId;
}

impl<'a, F: FilterNode<<CFGraph as GraphBase>::NodeId>> NodeCount for &'a SubGraph<'a, F> {
    fn node_count(self: &Self) -> usize {
        self.0
            .node_references()
            .filter(|(it, _)| self.0 .1.include_node(*it))
            .count()
    }
}

impl<'a, F: Copy + FilterNode<<CFGraph as GraphBase>::NodeId>> GraphRef for SubGraph<'a, F> {}

impl<'a, F: FilterNode<<CFGraph as GraphBase>::NodeId>> IntoNeighbors for &'a SubGraph<'a, F> {
    type Neighbors = <&'a FilteredCFGraph<'a, F> as IntoNeighbors>::Neighbors;

    fn neighbors(self, a: Self::NodeId) -> Self::Neighbors {
        self.0.neighbors(a)
    }
}

impl<'a, F: FilterNode<<CFGraph as GraphBase>::NodeId>> IntoNeighborsDirected
    for &'a SubGraph<'a, F>
{
    type NeighborsDirected =
        <&'a FilteredCFGraph<'a, F> as IntoNeighborsDirected>::NeighborsDirected;

    fn neighbors_directed(self, n: Self::NodeId, d: Direction) -> Self::NeighborsDirected {
        self.0.neighbors_directed(n, d)
    }
}
