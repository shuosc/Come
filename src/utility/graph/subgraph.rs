use std::{
    fmt::{self, Debug},
    vec,
};

use itertools::Itertools;
use petgraph::{
    adj::NodeIndex,
    graph::{DiGraph, EdgeIndex},
    graphmap::NeighborsDirected,
    visit::{
        Data, EdgeFiltered, EdgeFilteredNeighbors, EdgeRef, FilterEdge, FilterNode, GraphBase,
        GraphRef, IntoEdgeReferences, IntoEdges, IntoEdgesDirected, IntoNeighbors,
        IntoNeighborsDirected, IntoNodeIdentifiers, IntoNodeReferences, NodeCount, NodeFiltered,
        Visitable,
    },
    Direction,
};

pub type CFGraph = DiGraph<(), (), usize>;

#[derive(Debug, Clone)]
pub struct CFSubGraph<'a> {
    pub graph: &'a CFGraph,
    pub nodes: Vec<<CFGraph as GraphBase>::NodeId>,
    pub edges: Vec<<CFGraph as GraphBase>::EdgeId>,
}

impl<'a> CFSubGraph<'a> {
    pub fn new(
        graph: &'a CFGraph,
        nodes: impl IntoIterator<Item = <CFGraph as GraphBase>::NodeId>,
        edges: impl IntoIterator<Item = <CFGraph as GraphBase>::EdgeId>,
    ) -> Self {
        Self {
            graph,
            nodes: nodes.into_iter().collect(),
            edges: edges.into_iter().collect(),
        }
    }

    pub fn find_edge(
        &self,
        a: <CFGraph as GraphBase>::NodeId,
        b: <CFGraph as GraphBase>::NodeId,
    ) -> Option<<CFGraph as GraphBase>::EdgeId> {
        self.graph
            .find_edge(a, b)
            .filter(|it| self.edges.contains(it))
    }
}

impl<'a> GraphBase for CFSubGraph<'a> {
    type EdgeId = <CFGraph as GraphBase>::EdgeId;
    type NodeId = <CFGraph as GraphBase>::NodeId;
}

impl<'a> NodeCount for &'a CFSubGraph<'a> {
    fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

impl<'a> IntoNeighbors for &'a CFSubGraph<'a> {
    type Neighbors = vec::IntoIter<Self::NodeId>;

    fn neighbors(self, a: Self::NodeId) -> Self::Neighbors {
        let filtered_nodes = NodeFiltered::from_fn(self.graph, |n| self.nodes.contains(&n));
        let filtered_edges =
            EdgeFiltered::from_fn(&filtered_nodes, |e| self.edges.contains(&e.id()));
        filtered_nodes.neighbors(a).collect_vec().into_iter()
    }
}

impl<'a> IntoNeighborsDirected for &'a CFSubGraph<'a> {
    type NeighborsDirected = vec::IntoIter<Self::NodeId>;

    fn neighbors_directed(self, n: Self::NodeId, d: Direction) -> Self::NeighborsDirected {
        let filtered_nodes = NodeFiltered::from_fn(self.graph, |n| self.nodes.contains(&n));
        let filtered_edges =
            EdgeFiltered::from_fn(&filtered_nodes, |e| self.edges.contains(&e.id()));
        filtered_nodes
            .neighbors_directed(n, d)
            .collect_vec()
            .into_iter()
    }
}

impl<'a> Data for &'a CFSubGraph<'a> {
    type NodeWeight = <&'a CFGraph as Data>::NodeWeight;
    type EdgeWeight = <&'a CFGraph as Data>::EdgeWeight;
}

impl<'a> IntoEdgeReferences for &'a CFSubGraph<'a> {
    type EdgeRef = <&'a CFGraph as IntoEdgeReferences>::EdgeRef;

    type EdgeReferences = vec::IntoIter<Self::EdgeRef>;

    fn edge_references(self) -> Self::EdgeReferences {
        let filtered_nodes = NodeFiltered::from_fn(self.graph, |n| self.nodes.contains(&n));
        let filtered_edges =
            EdgeFiltered::from_fn(&filtered_nodes, |e| self.edges.contains(&e.id()));
        filtered_edges.edge_references().collect_vec().into_iter()
    }
}

impl<'a> IntoNodeIdentifiers for &'a CFSubGraph<'a> {
    type NodeIdentifiers = vec::IntoIter<Self::NodeId>;

    fn node_identifiers(self) -> Self::NodeIdentifiers {
        let filtered_nodes = NodeFiltered::from_fn(self.graph, |n| self.nodes.contains(&n));
        filtered_nodes.node_identifiers().collect_vec().into_iter()
    }
}

impl<'a> IntoEdges for &'a CFSubGraph<'a> {
    type Edges = vec::IntoIter<Self::EdgeRef>;

    fn edges(self, a: Self::NodeId) -> Self::Edges {
        let filtered_nodes = NodeFiltered::from_fn(self.graph, |n| self.nodes.contains(&n));
        let filtered_edges =
            EdgeFiltered::from_fn(&filtered_nodes, |e| self.edges.contains(&e.id()));
        filtered_edges.edges(a).collect_vec().into_iter()
    }
}

impl<'a> IntoEdgesDirected for &'a CFSubGraph<'a> {
    type EdgesDirected = vec::IntoIter<Self::EdgeRef>;

    fn edges_directed(self, a: Self::NodeId, dir: Direction) -> Self::EdgesDirected {
        let filtered_nodes = NodeFiltered::from_fn(self.graph, |n| self.nodes.contains(&n));
        let filtered_edges =
            EdgeFiltered::from_fn(&filtered_nodes, |e| self.edges.contains(&e.id()));
        filtered_edges
            .edges_directed(a, dir)
            .collect_vec()
            .into_iter()
    }
}

impl<'a> Visitable for &'a CFSubGraph<'a> {
    type Map = <CFGraph as Visitable>::Map;

    fn visit_map(&self) -> Self::Map {
        let filtered_nodes = NodeFiltered::from_fn(self.graph, |n| self.nodes.contains(&n));
        let filtered_edges =
            EdgeFiltered::from_fn(&filtered_nodes, |e| self.edges.contains(&e.id()));
        filtered_edges.visit_map()
    }

    fn reset_map(&self, map: &mut Self::Map) {
        let filtered_nodes = NodeFiltered::from_fn(self.graph, |n| self.nodes.contains(&n));
        let filtered_edges =
            EdgeFiltered::from_fn(&filtered_nodes, |e| self.edges.contains(&e.id()));
        filtered_edges.reset_map(map)
    }
}
