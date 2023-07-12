use come::utility::graph::dominance_frontiers;
use petgraph::{
    algo::dominators::{simple_fast, Dominators},
    prelude::*,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ControlFlowGraph {
    graph: DiGraph<(), (), u32>,
}

#[wasm_bindgen]
pub struct SimpleEdge(pub u32, pub u32);

#[wasm_bindgen]
impl ControlFlowGraph {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            graph: DiGraph::default(),
        }
    }

    pub fn add_edge(&mut self, a: u32, b: u32) {
        self.graph.extend_with_edges(&[(a, b)])
    }

    pub fn dominator_relation(&self) -> Option<DominatorRelation> {
        if self.graph.edge_count() != 0 {
            Some(DominatorRelation {
                dominators: simple_fast(&self.graph, 0.into()),
            })
        } else {
            None
        }
    }

    pub fn edges(&self) -> Vec<JsValue> {
        self.graph
            .edge_references()
            .map(|it| SimpleEdge(it.source().index() as _, it.target().index() as _))
            .map(JsValue::from)
            .collect()
    }
}

#[wasm_bindgen]
pub struct DominatorRelation {
    dominators: Dominators<NodeIndex>,
}

#[wasm_bindgen]
impl DominatorRelation {
    pub fn dominated_by(&self, node: u32) -> Vec<u32> {
        self.dominators
            .dominators(node.into())
            .map(|it| it.map(|it| it.index() as u32).collect())
            .unwrap_or_default()
    }
    pub fn immediately_dominates(&self, node: u32) -> Vec<u32> {
        self.dominators
            .immediately_dominated_by(node.into())
            .map(|it| it.index() as u32)
            .collect()
    }
    fn dominates_calculate(&self, visiting: u32, visited: &mut Vec<u32>) {
        if visited.contains(&visiting) {
            return;
        }
        visited.push(visiting);
        let mut imm_dominates: Vec<u32> = self.immediately_dominates(visiting);
        imm_dominates.retain(|it| !visited.contains(it));
        for it in imm_dominates {
            self.dominates_calculate(it, visited);
        }
    }
    pub fn dominates(&self, node: u32) -> Vec<u32> {
        let mut visited = Vec::new();
        self.dominates_calculate(node, &mut visited);
        visited
    }
    pub fn dominance_frontiers(&self, graph: &ControlFlowGraph, node: u32) -> Vec<u32> {
        dominance_frontiers(&self.dominators, &graph.graph)
            .get(&node.into())
            .map(|it| it.iter().map(|it| it.index() as u32).collect())
            .unwrap_or_default()
    }
}
