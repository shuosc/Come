use std::{collections::HashMap, fmt};

use itertools::Itertools;
use petgraph::{
    algo::all_simple_paths,
    prelude::NodeIndex,
    stable_graph::StableDiGraph,
    visit::{depth_first_search, DfsEvent, DfsPostOrder, EdgeRef},
    Direction,
};
use serde::{Deserialize, Serialize};

use super::BindedControlFlowGraph;

mod selector;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FoldedCFG {
    graph: StableDiGraph<RegionNode, Vec<(usize, usize)>, usize>,
    entry: NodeIndex<usize>,
}

impl fmt::Display for FoldedCFG {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(e) = self.graph.node_weight(self.entry) {
            write!(f, "{}", e)?;
        } else {
            write!(f, "??")?;
        }
        for n in self.graph.node_indices() {
            if n != self.entry {
                write!(f, ", {}", self.graph[n])?;
            }
        }
        Ok(())
    }
}

fn back_edges(c: &FoldedCFG) -> Vec<(NodeIndex<usize>, NodeIndex<usize>)> {
    let mut result = Vec::new();
    depth_first_search(&c.graph, Some(c.entry), |event| {
        if let DfsEvent::BackEdge(from, to) = event {
            result.push((from, to))
        }
    });
    result
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct If {
    content: FoldedCFG,
    on_then: NodeIndex<usize>,
    on_else: Option<NodeIndex<usize>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum RegionNode {
    Single(usize),
    Loop(FoldedCFG),
    Block(FoldedCFG),
    If(If),
}

impl fmt::Display for RegionNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegionNode::Single(bb_index) => write!(f, "{bb_index}"),
            RegionNode::Loop(content) => {
                write!(f, "loop({}", content.graph[content.entry])?;
                for n in content.graph.node_indices() {
                    if n != content.entry {
                        write!(f, ", {}", content.graph[n])?;
                    }
                }
                write!(f, ")")
            }
            RegionNode::Block(content) => {
                write!(f, "block({}", content.graph[content.entry])?;
                for n in content.graph.node_indices() {
                    if n != content.entry {
                        write!(f, ", {}", content.graph[n])?;
                    }
                }
                write!(f, ")")
            }
            RegionNode::If(content) => {
                write!(
                    f,
                    "if ({}) then ",
                    content.content.graph[content.content.entry]
                )?;
                if let Some(nn) = content.content.graph.node_weight(content.on_then) {
                    write!(f, "({})", nn)?;
                } else {
                    write!(f, "(??{})", content.on_then.index())?;
                }
                if let Some(on_else) = content.on_else {
                    if let Some(nn) = content.content.graph.node_weight(on_else) {
                        write!(f, " else ({})", nn)?;
                    } else {
                        write!(f, " else (??{})", on_else.index())?;
                    }
                }
                Ok(())
            }
        }
    }
}

impl FoldedCFG {
    fn fold_acyclic(
        content: &mut FoldedCFG,
        cfg: &BindedControlFlowGraph,
        node: NodeIndex<usize>,
    ) -> bool {
        let nodes = single_entry_single_exit_nodes(content, node);
        if nodes.len() >= 2 {
            fold_block(&nodes, content)
        } else if content
            .graph
            .neighbors_directed(node, Direction::Outgoing)
            .count()
            == 2
        {
            fold_if_else(content, node, cfg)
        } else {
            // todo!("switch ... etc")
            false
        }
    }

    fn fold_cyclic(content: &mut FoldedCFG, node: NodeIndex<usize>) -> bool {
        let backedges = back_edges(content);
        let backedges_into_node = backedges.into_iter().filter(|(_, to)| *to == node);
        let pathes_into_node = backedges_into_node.map(|(backedge_source, _)| {
            all_simple_paths::<Vec<_>, _>(&content.graph, node, backedge_source, 0, None)
                .flatten()
                .collect_vec()
        });
        let smallest_loop_content = pathes_into_node.min_by(|a, b| Ord::cmp(&a.len(), &b.len()));
        if let Some(mut smallest_loop_content) = smallest_loop_content {
            smallest_loop_content.sort();
            smallest_loop_content.dedup();
            let mut loop_subgraph = FoldedCFG::default();
            let mut node_index_map = HashMap::new(); // id in old => id in new
            for loop_content_index in &smallest_loop_content {
                let loop_content_node = &content.graph[*loop_content_index];
                node_index_map.insert(
                    loop_content_index,
                    loop_subgraph.graph.add_node(loop_content_node.clone()),
                );
            }
            loop_subgraph.entry = node_index_map[&node];
            let mut global_incoming_edges: HashMap<_, Vec<_>> = HashMap::new(); // source => weight
            let mut global_outgoing_edges: HashMap<_, Vec<_>> = HashMap::new();
            for loop_content_index in &smallest_loop_content {
                let incoming_edges = content
                    .graph
                    .edges_directed(*loop_content_index, Direction::Incoming);
                let (intern_edges, extern_edges) = incoming_edges
                    .partition::<Vec<_>, _>(|it| node_index_map.contains_key(&it.source()));
                for intern_edge in intern_edges {
                    let intern_from = node_index_map.get(&intern_edge.source()).unwrap();
                    let intern_target = node_index_map.get(&intern_edge.target()).unwrap();
                    loop_subgraph.graph.add_edge(
                        *intern_from,
                        *intern_target,
                        intern_edge.weight().clone(),
                    );
                }
                for extern_edge in extern_edges {
                    global_incoming_edges
                        .entry(extern_edge.source())
                        .or_default()
                        .extend(extern_edge.weight().clone());
                }

                let outgoing_edges = content
                    .graph
                    .edges_directed(*loop_content_index, Direction::Outgoing);
                let (_intern_edges, extern_edges) = outgoing_edges
                    .partition::<Vec<_>, _>(|it| node_index_map.contains_key(&it.target()));
                // no need to handle `intern_edges`, because they will be handled of `incoming_edges` on the other side
                for extern_edge in extern_edges {
                    global_outgoing_edges
                        .entry(extern_edge.target())
                        .or_default()
                        .extend(extern_edge.weight().clone());
                }
            }
            let new_node = content.graph.add_node(RegionNode::Loop(loop_subgraph));
            for (from, weight) in global_incoming_edges {
                content.graph.add_edge(from, new_node, weight);
            }
            for (to, weight) in global_outgoing_edges {
                content.graph.add_edge(new_node, to, weight);
            }
            for node_to_remove in smallest_loop_content.iter().rev() {
                content.graph.remove_node(*node_to_remove);
            }
            if smallest_loop_content.contains(&content.entry) {
                content.entry = new_node;
            }
            true
        } else {
            false
        }
    }

    pub fn structural_analysis(mut self, cfg: &BindedControlFlowGraph) -> FoldedCFG {
        'outer: loop {
            let mut result = self.clone();
            let mut dfs = DfsPostOrder::new(&self.graph, self.entry);
            while let Some(node) = dfs.next(&self.graph) {
                if Self::fold_acyclic(&mut result, cfg, node) {
                    self = result;
                    continue 'outer;
                }
                if Self::fold_cyclic(&mut result, node) {
                    self = result;
                    continue 'outer;
                }
            }
            break;
        }
        self
    }
    pub fn from_control_flow_graph(graph: &BindedControlFlowGraph) -> FoldedCFG {
        let graph_ = graph.graph();
        let result = graph_.map(
            |node_index, _| RegionNode::Single(node_index.index()),
            |edge_id, _| {
                let (from, to) = graph_.edge_endpoints(edge_id).unwrap();
                vec![(from.index(), to.index())]
            },
        );
        FoldedCFG {
            graph: result.into(),
            entry: 0.into(),
        }
    }
}

fn fold_if_else(
    region_content: &mut FoldedCFG,
    condition: NodeIndex<usize>,
    cfg: &BindedControlFlowGraph,
) -> bool {
    let (outgoing_node_a, outgoing_node_b) =
        outgoing_nodes_from_condition(region_content, condition, cfg);
    let subregion_node = &region_content.graph[condition];
    let subregion_a = &region_content.graph[outgoing_node_a].clone();
    let mut new_region_content = FoldedCFG {
        graph: StableDiGraph::with_capacity(3, 2),
        entry: 0.into(),
    };
    let inserted_subregion_node_index = new_region_content.graph.add_node(subregion_node.clone());
    new_region_content.entry = inserted_subregion_node_index;
    let after_a = region_content
        .graph
        .neighbors_directed(outgoing_node_a, Direction::Outgoing)
        .collect_vec();
    let after_b = region_content
        .graph
        .neighbors_directed(outgoing_node_b, Direction::Outgoing)
        .collect_vec();
    let edges_into_node = region_content
        .graph
        .edges_directed(condition, Direction::Incoming)
        .map(|it| (it.source(), it.weight().clone()))
        .collect_vec();
    if after_a.contains(&outgoing_node_b) {
        // if, without an else
        // the content should only contains the condition (`node`) and the then part (`a`)
        let inserted_subregion_a_index = new_region_content.graph.add_node(subregion_a.clone());
        let edge_condition_to_a = region_content
            .graph
            .find_edge(condition, outgoing_node_a)
            .unwrap();
        new_region_content.graph.add_edge(
            inserted_subregion_node_index,
            inserted_subregion_a_index,
            region_content.graph[edge_condition_to_a].clone(),
        );
        let new_if = If {
            content: new_region_content,
            on_then: inserted_subregion_a_index,
            on_else: None,
        };
        // create a new node containing all nodes in `nodes`
        let new_node = region_content.graph.add_node(RegionNode::If(new_if));
        // redirect in edges
        for edge_into_node in edges_into_node {
            region_content
                .graph
                .add_edge(edge_into_node.0, new_node, edge_into_node.1.clone());
        }
        // redirect out edges
        let condition_to_b = region_content
            .graph
            .find_edge(condition, outgoing_node_b)
            .unwrap();
        let condition_to_b = &region_content.graph[condition_to_b];
        let a_to_b = region_content
            .graph
            .find_edge(outgoing_node_a, outgoing_node_b)
            .unwrap();
        let a_to_b = &region_content.graph[a_to_b];
        let _new_edge = region_content.graph.add_edge(
            new_node,
            outgoing_node_b,
            Iterator::chain(condition_to_b.iter(), a_to_b)
                .cloned()
                .collect(),
        );
        // remove nodes in `nodes`
        region_content
            .graph
            .retain_nodes(|_, n| n != condition && n != outgoing_node_a);
        true
    } else if after_a.len() == 1 && after_a == after_b {
        let c = after_a[0];
        // if, with an else
        let subregion_b = &region_content.graph[outgoing_node_b].clone();
        let inserted_subregion_a_index = new_region_content.graph.add_node(subregion_a.clone());
        let inserted_subregion_b_index = new_region_content.graph.add_node(subregion_b.clone());
        let edge_condition_to_a = region_content
            .graph
            .find_edge(condition, outgoing_node_a)
            .unwrap();
        let edge_condition_to_b = region_content
            .graph
            .find_edge(condition, outgoing_node_b)
            .unwrap();
        new_region_content.graph.add_edge(
            inserted_subregion_node_index,
            inserted_subregion_a_index,
            region_content.graph[edge_condition_to_a].clone(),
        );
        new_region_content.graph.add_edge(
            inserted_subregion_node_index,
            inserted_subregion_b_index,
            region_content.graph[edge_condition_to_b].clone(),
        );
        let new_if = If {
            content: new_region_content,
            on_then: inserted_subregion_a_index,
            on_else: Some(inserted_subregion_b_index),
        };
        // create a new node containing all nodes in `nodes`
        let new_node = region_content.graph.add_node(RegionNode::If(new_if));
        // redirect in edges
        for edge_into_node in edges_into_node {
            region_content
                .graph
                .add_edge(edge_into_node.0, new_node, edge_into_node.1.clone());
        }
        // redirect out edges
        let (a_to_c, b_to_c) = region_content
            .graph
            .edges_directed(outgoing_node_a, Direction::Outgoing)
            .chain(
                region_content
                    .graph
                    .edges_directed(outgoing_node_b, Direction::Outgoing),
            )
            .collect_tuple()
            .unwrap();
        let new_weight = Iterator::chain(a_to_c.weight().iter(), b_to_c.weight())
            .cloned()
            .collect();
        let _new_edge = region_content.graph.add_edge(new_node, c, new_weight);
        // remove nodes in `nodes`
        region_content
            .graph
            .retain_nodes(|_, n| n != condition && n != outgoing_node_a && n != outgoing_node_b);
        if condition == region_content.entry {
            region_content.entry = new_node;
        }
        true
    } else {
        false
    }
}

fn outgoing_nodes_from_condition(
    region_content: &FoldedCFG,
    condition: NodeIndex<usize>,
    cfg: &BindedControlFlowGraph,
) -> (NodeIndex<usize>, NodeIndex<usize>) {
    let (mut outgoing_node_a, mut outgoing_node_b) = region_content
        .graph
        .neighbors_directed(condition, Direction::Outgoing)
        .collect_tuple()
        .unwrap();
    if region_content
        .graph
        .neighbors_directed(outgoing_node_b, Direction::Outgoing)
        .contains(&outgoing_node_a)
    {
        // we make sure `outgoing_node_a` is at least as high as `outgoing_node_b`
        (outgoing_node_a, outgoing_node_b) = (outgoing_node_b, outgoing_node_a);
    } else {
        // we make sure `outgoing_node_a` is on the `then` part and `outgoing_node_b` is on the else part
        let edge_in_origin_graph_node_to_a = region_content
            .graph
            .edges_connecting(condition, outgoing_node_a)
            .exactly_one()
            .unwrap()
            .weight()
            .first()
            .unwrap();
        if !cfg.branch_direction(
            edge_in_origin_graph_node_to_a.0,
            edge_in_origin_graph_node_to_a.1,
        ) {
            // a is not on the "then" side, swap them
            (outgoing_node_a, outgoing_node_b) = (outgoing_node_b, outgoing_node_a);
        }
    }
    (outgoing_node_a, outgoing_node_b)
}

fn single_entry_single_exit_nodes(
    content: &FoldedCFG,
    node: NodeIndex<usize>,
) -> Vec<NodeIndex<usize>> {
    let mut result = Vec::new();
    let mut current_looking_at_node = node;
    // detect up
    while content
        .graph
        .neighbors_directed(current_looking_at_node, Direction::Incoming)
        .count()
        == 1
        && content
            .graph
            .neighbors_directed(current_looking_at_node, Direction::Outgoing)
            .count()
            == 1
    {
        result.push(current_looking_at_node);
        current_looking_at_node = content
            .graph
            .neighbors_directed(current_looking_at_node, Direction::Incoming)
            .exactly_one()
            .unwrap();
    }
    result.reverse();
    result.pop();
    current_looking_at_node = node;
    // detect down
    while content
        .graph
        .neighbors_directed(current_looking_at_node, Direction::Incoming)
        .count()
        == 1
        && content
            .graph
            .neighbors_directed(current_looking_at_node, Direction::Outgoing)
            .count()
            == 1
    {
        result.push(current_looking_at_node);
        current_looking_at_node = content
            .graph
            .neighbors_directed(current_looking_at_node, Direction::Outgoing)
            .exactly_one()
            .unwrap();
    }
    result
}

fn fold_block(nodes: &[NodeIndex<usize>], region_content: &mut FoldedCFG) -> bool {
    if nodes.len() < 2 {
        return false;
    }
    let mut new_subregion_content = FoldedCFG {
        graph: StableDiGraph::with_capacity(nodes.len(), nodes.len() - 1),
        entry: 0.into(),
    };
    let mut last_node_index = None;
    let mut last_inserted = None;
    for node in nodes {
        let sub_region_in_origin = region_content.graph[*node].clone();
        let current_inserted = new_subregion_content.graph.add_node(sub_region_in_origin);
        if let Some(last_inserted) = last_inserted {
            let last_node_index = last_node_index.unwrap();
            let origin_edge_weight = region_content
                .graph
                .edges_connecting(last_node_index, *node)
                .exactly_one()
                .unwrap()
                .weight();
            new_subregion_content.graph.add_edge(
                last_inserted,
                current_inserted,
                origin_edge_weight.clone(),
            );
        }
        last_node_index = Some(*node);
        last_inserted = Some(current_inserted);
    }
    let edges_into_the_new_node = region_content
        .graph
        .edges_directed(nodes[0], Direction::Incoming)
        .map(|e| (e.source(), e.target(), e.weight().clone()))
        .collect_vec();
    let edges_out_of_the_new_node = region_content
        .graph
        .edges_directed(*nodes.last().unwrap(), Direction::Outgoing)
        .map(|e| (e.source(), e.target(), e.weight().clone()))
        .collect_vec();
    // create a new node containing all nodes in `nodes`
    let new_node = region_content
        .graph
        .add_node(RegionNode::Block(new_subregion_content));
    // re-point all edges into the new node to the new node
    for edge_into_the_new_node in edges_into_the_new_node {
        region_content.graph.add_edge(
            edge_into_the_new_node.0,
            new_node,
            edge_into_the_new_node.2.clone(),
        );
    }
    // re-point all edges out of the new node from the new node
    for edge_out_of_the_new_node in edges_out_of_the_new_node {
        region_content.graph.add_edge(
            new_node,
            edge_out_of_the_new_node.1,
            edge_out_of_the_new_node.2.clone(),
        );
    }
    // remove nodes in `nodes`
    region_content
        .graph
        .retain_nodes(|_, n| !nodes.contains(&n));
    true
}

#[cfg(test)]
mod tests {
    use crate::{
        ir::{
            self,
            analyzer::{ControlFlowGraph, IsAnalyzer},
            function::test_util::*,
            FunctionDefinition,
        },
        utility::data_type,
    };

    use super::*;

    #[test]
    fn test_fold_cyclic_self() {
        let mut graph = StableDiGraph::default();
        let node_0 = graph.add_node(RegionNode::Single(0));
        graph.add_edge(node_0, node_0, vec![(0, 0)]);
        let mut graph = FoldedCFG {
            graph,
            entry: node_0,
        };

        FoldedCFG::fold_cyclic(&mut graph, 0.into());
        dbg!(graph);
    }
    #[test]
    fn test_fold_cyclic_two() {
        let mut graph = StableDiGraph::default();
        let node_0 = graph.add_node(RegionNode::Single(0));
        let node_1 = graph.add_node(RegionNode::Single(1));
        graph.add_edge(node_0, node_1, vec![(0, 1)]);
        graph.add_edge(node_1, node_0, vec![(1, 0)]);
        let mut graph = FoldedCFG {
            graph,
            entry: node_0,
        };
        dbg!(back_edges(&graph));

        FoldedCFG::fold_cyclic(&mut graph /*,cfg */, 0.into());
    }
    #[test]
    fn test_fold_cyclic_three() {
        let mut graph = StableDiGraph::default();
        let node_0 = graph.add_node(RegionNode::Single(0));
        let node_1 = graph.add_node(RegionNode::Single(1));
        let node_2 = graph.add_node(RegionNode::Single(2));
        let node_3 = graph.add_node(RegionNode::Single(3));
        graph.add_edge(node_0, node_1, vec![(0, 1)]);
        graph.add_edge(node_1, node_2, vec![(1, 2)]);
        graph.add_edge(node_2, node_0, vec![(2, 0)]);
        graph.add_edge(node_1, node_3, vec![(1, 3)]);
        graph.add_edge(node_2, node_3, vec![(2, 3)]);
        let mut graph = FoldedCFG {
            graph,
            entry: node_0,
        };

        FoldedCFG::fold_cyclic(&mut graph, 0.into());
        dbg!(graph);
    }
    #[test]
    fn test_fold_cyclic_three_2() {
        let mut graph = StableDiGraph::default();
        let node_0 = graph.add_node(RegionNode::Single(0));
        let node_1 = graph.add_node(RegionNode::Single(1));
        let node_2 = graph.add_node(RegionNode::Single(2));
        graph.add_edge(node_0, node_1, vec![(0, 1)]);
        graph.add_edge(node_1, node_0, vec![(1, 0)]);
        graph.add_edge(node_1, node_2, vec![(1, 2)]);
        graph.add_edge(node_2, node_0, vec![(2, 0)]);
        let mut graph = FoldedCFG {
            graph,
            entry: node_0,
        };

        FoldedCFG::fold_cyclic(&mut graph, 0.into());
        dbg!(graph);
    }

    #[test]
    fn test_fold_cyclic_four() {
        let mut graph = StableDiGraph::default();
        let node_0 = graph.add_node(RegionNode::Single(0));
        let node_1 = graph.add_node(RegionNode::Single(1));
        let node_2 = graph.add_node(RegionNode::Single(2));
        let node_3 = graph.add_node(RegionNode::Single(3));
        graph.add_edge(node_0, node_1, vec![(0, 1)]);
        graph.add_edge(node_1, node_0, vec![(1, 0)]);
        graph.add_edge(node_1, node_2, vec![(1, 2)]);
        graph.add_edge(node_2, node_0, vec![(2, 0)]);
        graph.add_edge(node_2, node_3, vec![(2, 3)]);
        graph.add_edge(node_3, node_0, vec![(3, 0)]);
        let mut graph = FoldedCFG {
            graph,
            entry: node_0,
        };
        FoldedCFG::fold_cyclic(&mut graph, 0.into());
        dbg!(graph);
    }

    #[test]
    fn test_fold_cyclic_seq() {
        let mut graph = StableDiGraph::default();
        let node_0 = graph.add_node(RegionNode::Single(0));
        let node_1 = graph.add_node(RegionNode::Single(1));
        let node_2 = graph.add_node(RegionNode::Single(2));
        let node_3 = graph.add_node(RegionNode::Single(3));
        let node_4 = graph.add_node(RegionNode::Single(4));
        graph.add_edge(node_0, node_1, vec![(0, 1)]);
        graph.add_edge(node_1, node_2, vec![(1, 2)]);
        graph.add_edge(node_2, node_3, vec![(2, 3)]);
        graph.add_edge(node_3, node_1, vec![(3, 1)]);
        graph.add_edge(node_3, node_4, vec![(3, 4)]);
        let mut graph = FoldedCFG {
            graph,
            entry: node_0,
        };
        FoldedCFG::fold_cyclic(&mut graph, node_1);
        dbg!(graph);
    }

    #[test]
    fn test_structual_analysis() {
        let function_definition = FunctionDefinition {
            header: ir::FunctionHeader {
                name: "f".to_string(),
                parameters: Vec::new(),
                return_type: data_type::Type::None,
            },
            content: vec![
                branch_block(0, 1, 2),
                jump_block(1, 3),
                jump_block(2, 6),
                jump_block(3, 4),
                jump_block(4, 5),
                branch_block(5, 3, 6),
                ret_block(6),
            ],
        };
        let control_flow_graph = ControlFlowGraph::new();
        let binded = control_flow_graph.bind(&function_definition);
        dbg!(&binded);
        let folded_cfg = FoldedCFG::from_control_flow_graph(&binded);
        let saed = FoldedCFG::structural_analysis(folded_cfg, &binded);
        println!("{}", saed);
    }

    #[test]
    fn test_structual_analysis2() {
        let function_definition = FunctionDefinition {
            header: ir::FunctionHeader {
                name: "f".to_string(),
                parameters: Vec::new(),
                return_type: data_type::Type::None,
            },
            content: vec![
                jump_block(0, 1),
                branch_block(1, 2, 3),
                jump_block(2, 3),
                ret_block(3),
            ],
        };
        let control_flow_graph = ControlFlowGraph::new();
        let binded = control_flow_graph.bind(&function_definition);
        let folded_cfg = FoldedCFG::from_control_flow_graph(&binded);
        let saed = FoldedCFG::structural_analysis(folded_cfg, &binded);
        println!("{}", saed);
    }

    #[test]
    fn test_structual_analysis3() {
        let function_definition = FunctionDefinition {
            header: ir::FunctionHeader {
                name: "f".to_string(),
                parameters: Vec::new(),
                return_type: data_type::Type::None,
            },
            content: vec![
                jump_block(0, 1),
                branch_block(1, 2, 6),
                branch_block(2, 3, 4),
                jump_block(3, 5),
                jump_block(5, 1),
                jump_block(4, 5),
                branch_block(6, 7, 8),
                jump_block(7, 9),
                jump_block(8, 9),
                ret_block(9),
            ],
        };
        let control_flow_graph = ControlFlowGraph::new();
        let binded = control_flow_graph.bind(&function_definition);
        let folded_cfg = FoldedCFG::from_control_flow_graph(&binded);
        let saed = FoldedCFG::structural_analysis(folded_cfg, &binded);
        println!("{}", saed);
    }

    #[test]
    fn test_structual_analysis4() {
        let code = r"fn test_condition(i32 %a, i32 %b) -> i32 {
          test_condition_entry:
            %a_0_addr = alloca i32
            store i32 %a, address %a_0_addr
            %b_0_addr = alloca i32
            store i32 %b, address %b_0_addr
            %1 = load i32 %a_0_addr
            %2 = load i32 %b_0_addr
            %0 = slt i32 %1, %2
            bne %0, 0, if_0_success, if_0_fail
          if_0_success:
            %3 = load i32 %a_0_addr
            ret %3
          if_0_fail:
            %4 = load i32 %b_0_addr
            ret %4
          if_0_end:
            ret
        }";
        let ir_code = ir::parse(code).unwrap().1;
        let f = ir_code.as_function_definition();
        let cfg = ControlFlowGraph::new();
        let cfg = cfg.bind(f);
        let folded = FoldedCFG::from_control_flow_graph(&cfg);
        let result = folded.structural_analysis(&cfg);
        dbg!(result);
    }
}
