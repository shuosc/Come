use std::mem;

use itertools::Itertools;
use petgraph::prelude::*;

use crate::ir::{
    analyzer::{IsAnalyzer, Loop},
    optimize::pass::fix_irreducible::FixIrreducible,
};

use super::IsPass;

pub struct TopologicalSort;

impl IsPass for TopologicalSort {
    fn run(&self, editor: &mut crate::ir::editor::Editor) {
        let analyzer = editor.analyzer.bind(&editor.content);
        let graph = analyzer.control_flow_graph();
        let loops = graph.loops();
        let graph = graph.graph();
        let content: Vec<_> = topological_order(graph, &loops)
            .into_iter()
            .map(|it| mem::take(&mut editor.content.content[it]))
            .collect();
        editor.content.content = content;
    }

    fn need(&self) -> Vec<super::Pass> {
        vec![FixIrreducible.into()]
    }

    fn invalidate(&self) -> Vec<super::Pass> {
        Vec::new()
    }
}

fn topological_order_dfs(
    graph: &DiGraph<(), (), usize>,
    top_level: &Loop,
    current_at: NodeIndex<usize>,
    result: &mut Vec<NodeIndex<usize>>,
) {
    if result.contains(&current_at) {
        return;
    }
    result.push(current_at);
    let in_loop = top_level.smallest_loop_node_in(current_at);
    assert!({
        if let Some(in_loop) = in_loop {
            if let Some(&root) = in_loop.entries.first() {
                let root: NodeIndex<usize> = root.into();
                result.contains(&root)
            } else {
                true
            }
        } else {
            true
        }
    });
    let mut to_visit = graph
        .neighbors_directed(current_at, Direction::Outgoing)
        .filter(|it| !result.contains(it))
        .collect::<Vec<_>>();
    to_visit.sort_unstable();
    to_visit.sort_by_cached_key(|to_visit_node| {
        let forward_of_to_visit_node =
            graph.neighbors_directed(*to_visit_node, Direction::Incoming);
        // first visit those nodes which current_at is the only parent of to_visit_node
        if forward_of_to_visit_node.count() == 1 {
            return 0;
        }
        // we should visit all nodes in this loop before the others
        if let Some(in_loop) = in_loop && in_loop.is_node_in(*to_visit_node) {
            return 1;
        }
        2
    });
    for to_visit_node in to_visit {
        topological_order_dfs(graph, top_level, to_visit_node, result);
    }
}

pub fn topological_order(graph: &DiGraph<(), (), usize>, top_level: &Loop) -> Vec<usize> {
    let mut order = vec![];
    topological_order_dfs(graph, top_level, 0.into(), &mut order);
    let mut order: Vec<usize> = order.into_iter().map(NodeIndex::index).collect();
    let exit_block_position = order.iter().position_max().unwrap();
    order.remove(exit_block_position);
    order
}

#[cfg(test)]
mod tests {
    use crate::{
        ir::{
            self,
            function::{basic_block::BasicBlock, test_util::*},
            optimize::pass::IsPass,
            statement::Ret,
            FunctionDefinition,
        },
        utility::data_type,
    };

    use super::TopologicalSort;

    #[test]
    fn test_topological_order() {
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
                    name: Some("bb9".to_string()),
                    content: vec![jump("bb11")],
                },
                BasicBlock {
                    name: Some("bb10".to_string()),
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
        let mut editor = ir::editor::Editor::new(function_definition);
        let pass = TopologicalSort;
        pass.run(&mut editor);
        assert_eq!(editor.content.content[0].name, Some("bb0".to_string()));
        let bb1_pos = editor
            .content
            .content
            .iter()
            .position(|it| it.name == Some("bb1".to_string()))
            .unwrap();
        let bb2_pos = editor
            .content
            .content
            .iter()
            .position(|it| it.name == Some("bb2".to_string()))
            .unwrap();
        let bb5_pos = editor
            .content
            .content
            .iter()
            .position(|it| it.name == Some("bb5".to_string()))
            .unwrap();
        assert!(bb1_pos < bb2_pos);
        assert!(bb2_pos < bb5_pos);
        let bb3_pos = editor
            .content
            .content
            .iter()
            .position(|it| it.name == Some("bb3".to_string()))
            .unwrap();
        assert_eq!(bb2_pos + 1, bb3_pos);
    }
}
