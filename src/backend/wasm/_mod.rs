use std::fmt::{self, Debug, Display};

use itertools::Itertools;
use petgraph::{dot::Dot, Direction};

use crate::{
    ir::{
        self,
        analyzer::{self, Analyzer, BindedAnalyzer, IsAnalyzer, LoopContent},
        statement::IRStatement,
    },
    utility::graph,
};
#[derive(PartialEq, Clone)]
pub struct ControlFlow {
    content: Vec<ControlFlowContent>,
}

impl Debug for ControlFlow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.content
            .iter()
            .map(|item| write!(f, "{:?}", item))
            .collect()
    }
}

impl ControlFlow {
    pub fn node_count(&self) -> usize {
        self.content.iter().map(|it| it.node_count()).sum()
    }

    pub fn contains(&self, node: usize) -> bool {
        for content_item in &self.content {
            match content_item {
                ControlFlowContent::Block(x) => {
                    if x.contains(node) {
                        return true;
                    }
                }
                ControlFlowContent::BrIf(x) => {
                    if *x == node {
                        return true;
                    }
                }
                ControlFlowContent::If(x, y) => {
                    if x.contains(node) {
                        return true;
                    }
                    if let Some(y) = y {
                        if y.contains(node) {
                            return true;
                        }
                    }
                }
                ControlFlowContent::Loop(x) => {
                    if x.contains(node) {
                        return true;
                    }
                }
                ControlFlowContent::Node(x) => {
                    if *x == node {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn first_node(&self) -> usize {
        self.content.first().unwrap().first_node()
    }
}

#[derive(PartialEq, Clone)]
pub enum ControlFlowContent {
    Block(Box<ControlFlow>),
    BrIf(usize),
    If(Box<ControlFlow>, Option<ControlFlow>),
    Loop(Box<ControlFlow>),
    Node(usize),
}

impl Debug for ControlFlowContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            ControlFlowContent::Block(cf) => {
                write!(f, "Block {{ {:?}}} ", cf)?;
            }
            ControlFlowContent::BrIf(i) => {
                write!(f, "BrIf {:?}", i)?;
            }
            ControlFlowContent::If(cf1, cf2) => {
                write!(f, "If {{ {:?}}} ", cf1)?;
                if let Some(cf2) = cf2 {
                    write!(f, "Else {{ {:?}}} ", cf2)?;
                }
            }
            ControlFlowContent::Loop(cf) => {
                write!(f, "Loop {{ {:?}}} ", cf)?;
            }
            ControlFlowContent::Node(i) => {
                write!(f, "{:?} ", i)?;
            }
        }
        Ok(())
    }
}

impl ControlFlowContent {
    pub fn node_count(&self) -> usize {
        match self {
            ControlFlowContent::Block(x) => x.node_count(),
            ControlFlowContent::BrIf(_) => 0,
            ControlFlowContent::If(x, y) => {
                x.node_count() + y.as_ref().map(|it| it.node_count()).unwrap_or(0)
            }
            ControlFlowContent::Loop(x) => x.node_count(),
            ControlFlowContent::Node(_) => 1,
        }
    }

    pub fn contains(&self, node: usize) -> bool {
        match self {
            ControlFlowContent::Block(x) => x.contains(node),
            ControlFlowContent::BrIf(x) => *x == node,
            ControlFlowContent::If(x, y) => {
                x.contains(node) || y.as_ref().map(|it| it.contains(node)).unwrap_or(false)
            }
            ControlFlowContent::Loop(x) => x.contains(node),
            ControlFlowContent::Node(x) => *x == node,
        }
    }

    pub fn first_node(&self) -> usize {
        match self {
            ControlFlowContent::Block(x) => x.first_node(),
            ControlFlowContent::BrIf(_) => panic!(),
            ControlFlowContent::If(x, _) => x.first_node(),
            ControlFlowContent::Loop(x) => x.first_node(),
            ControlFlowContent::Node(x) => *x,
        }
    }
}

fn fold_loop(current: &mut Vec<ControlFlowContent>, loop_item: &analyzer::Loop) {
    let entry = loop_item.entries.first().unwrap();
    let slice_start = current
        .iter()
        .position(|it| {
            if let ControlFlowContent::Node(x) = it {
                x == entry
            } else {
                false
            }
        })
        .unwrap();
    let slice_end = slice_start + loop_item.node_count();
    let mut removed = current.drain(slice_start..slice_end).collect_vec();
    let sub_loops = loop_item.content.iter().filter_map(|it| {
        if let LoopContent::SubLoop(subloop) = it {
            Some(subloop)
        } else {
            None
        }
    });
    for sub_loop in sub_loops {
        fold_loop(&mut removed, &sub_loop);
    }
    let new_loop_item = ControlFlowContent::Loop(Box::new(ControlFlow { content: removed }));
    current.insert(slice_start, new_loop_item);
}

fn nest_branch(current: &mut Vec<ControlFlowContent>, analyzer: &BindedAnalyzer) {
    let graph = analyzer.control_flow_graph();
    let mut current_index = 0;
    while current_index < current.len() {
        let current_item = &current[current_index];
        match current_item {
            ControlFlowContent::Node(node) => {
                let dominates = graph.dominates(*node);
                let predecessors = graph
                    .graph()
                    .neighbors_directed((*node).into(), Direction::Incoming)
                    .filter(|it| !dominates.contains(&it.index()))
                    .collect_vec();
                if predecessors.len() == 1 {
                    // try nesting `node` and nodes dominated by `node`
                    // into branch started by `predecessor`
                    let predecessor = predecessors[0];
                    println!("nest {} into {:?}", node, predecessor);
                    let predecessor_last =
                        analyzer.bind_on.content[predecessor.index()].content.last();
                    if let Some(IRStatement::Branch(branch)) = predecessor_last {
                        println!("{:?} is already a branch", predecessor);
                        let predecessor_index = current
                            .iter()
                            .position(|it| it.contains(predecessor.index()))
                            .unwrap();
                        let nodes_dominated_by_current_node = graph
                            .dominates(*node)
                            .into_iter()
                            .filter(|it| it != node)
                            .collect_vec();
                        dbg!(&nodes_dominated_by_current_node);
                        let initial_index = current_index;
                        current_index += 1;
                        while nodes_dominated_by_current_node
                            .iter()
                            .find(|it| current[current_index].contains(**it))
                            .is_some()
                        {
                            current_index += 1;
                        }
                        let mut content = current.drain(initial_index..current_index).collect();
                        nest_branch(&mut content, analyzer);
                        let after_predecessor = current.get_mut(predecessor_index + 1);
                        // FIXME: is it in order?
                        if let Some(ControlFlowContent::If(_, else_part)) = after_predecessor {
                            *else_part = Some(ControlFlow { content })
                        } else {
                            current.insert(
                                initial_index,
                                ControlFlowContent::If(Box::new(ControlFlow { content }), None),
                            );
                        }
                    } else {
                        current_index += 1;
                    }
                } else {
                    current_index += 1;
                }
            }
            ControlFlowContent::Loop(loop_item) => {
                let first = loop_item.first_node();
                let dominates = graph.dominates(first);
                let predecessors = graph
                    .graph()
                    .neighbors_directed(first.into(), Direction::Incoming)
                    .filter(|it| !dominates.contains(&it.index()))
                    .collect_vec();
                if predecessors.len() == 1 {
                    // try nesting `loop_item` and nodes dominated by `loop_item`
                    // into branch started by `predecessor`
                    let predecessor = predecessors[0];
                    println!("nest {:?} into {:?}", loop_item, predecessor);
                    let predecessor_last_statement =
                        analyzer.bind_on.content[predecessor.index()].content.last();
                    if let Some(IRStatement::Branch(branch)) = predecessor_last_statement {
                        let predecessor_index = current
                            .iter()
                            .position(|it| it.contains(predecessor.index()))
                            .unwrap();
                        let mut content = vec![current.remove(current_index)];
                        nest_branch(&mut content, analyzer);
                        let after_predecessor = current.get_mut(predecessor_index + 1);
                        if let Some(ControlFlowContent::If(_, else_part)) = after_predecessor {
                            *else_part = Some(ControlFlow { content });
                        } else {
                            current.insert(
                                current_index,
                                ControlFlowContent::If(Box::new(ControlFlow { content }), None),
                            );
                        }
                    } else {
                        current_index += 1;
                    }
                } else {
                    current_index += 1;
                }
            }
            ControlFlowContent::Block(_)
            | ControlFlowContent::BrIf(_)
            | ControlFlowContent::If(_, _) => current_index += 1,
        }
        dbg!(&current);
    }
}

// function must be reduciable and in correct topo order
fn generate_control_flow(
    function: &ir::FunctionDefinition,
    analyzer: &Analyzer,
) -> Vec<ControlFlowContent> {
    let binded_analyzer = analyzer.bind(function);
    let mut current = function
        .content
        .iter()
        .enumerate()
        .map(|(index, _)| ControlFlowContent::Node(index))
        .collect_vec();
    fold_all_loops(binded_analyzer, &mut current);
    let binded_analyzer = analyzer.bind(function);
    nest_branch(&mut current, &binded_analyzer);
    current
}

fn fold_all_loops(
    binded_analyzer: analyzer::BindedAnalyzer,
    initial: &mut Vec<ControlFlowContent>,
) {
    let analyzed_loops = binded_analyzer.control_flow_graph().loops();
    let sub_loops = analyzed_loops.content.iter().filter_map(|it| {
        if let LoopContent::SubLoop(subloop) = it {
            Some(subloop)
        } else {
            None
        }
    });
    for sub_loop in sub_loops {
        fold_loop(initial, &sub_loop);
    }
}

#[test]
fn test_fold() {
    use crate::{
        ir::{
            function::{basic_block::BasicBlock, test_util::*},
            statement::Ret,
            FunctionDefinition,
        },
        utility::data_type,
    };
    let function_definition = FunctionDefinition {
        header: ir::FunctionHeader {
            name: "f".to_string(),
            parameters: Vec::new(),
            return_type: data_type::Type::None,
        },
        content: vec![
            BasicBlock {
                name: Some("bb0".to_string()),
                content: vec![branch("bb1", "bb7")],
            },
            BasicBlock {
                name: Some("bb1".to_string()),
                content: vec![jump("bb2")],
            },
            BasicBlock {
                name: Some("bb2".to_string()),
                content: vec![jump("bb3")],
            },
            BasicBlock {
                name: Some("bb3".to_string()),
                content: vec![jump("bb4")],
            },
            BasicBlock {
                name: Some("bb4".to_string()),
                content: vec![branch("bb5", "bb6")],
            },
            BasicBlock {
                name: Some("bb5".to_string()),
                content: vec![jump("bb2")],
            },
            BasicBlock {
                name: Some("bb6".to_string()),
                content: vec![branch("bb1", "bb14")],
            },
            BasicBlock {
                name: Some("bb7".to_string()),
                content: vec![branch("bb8", "bb9")],
            },
            BasicBlock {
                name: Some("bb8".to_string()),
                content: vec![jump("bb10")],
            },
            BasicBlock {
                name: Some("bb10".to_string()),
                content: vec![jump("bb11")],
            },
            BasicBlock {
                name: Some("bb9".to_string()),
                content: vec![jump("bb11")],
            },
            BasicBlock {
                name: Some("bb11".to_string()),
                content: vec![branch("bb12", "bb13")],
            },
            BasicBlock {
                name: Some("bb12".to_string()),
                content: vec![jump("bb13")],
            },
            BasicBlock {
                name: Some("bb13".to_string()),
                content: vec![jump("bb14")],
            },
            BasicBlock {
                name: Some("bb14".to_string()),
                content: vec![jump("bb15")],
            },
            BasicBlock {
                name: Some("bb15".to_string()),
                content: vec![Ret { value: None }.into()],
            },
        ],
    };
    let analyzer = Analyzer::default();
    let graph = analyzer.bind(&function_definition);
    let binding = graph.control_flow_graph();
    let graph = binding.graph();
    println!("{:?}", Dot::new(&graph));
    let result = generate_control_flow(&function_definition, &analyzer);
    dbg!(result);
}

// ABC
// ---
// --