use itertools::{Either, Itertools};
use std::{
    collections::VecDeque,
    fmt,
    iter::zip,
    mem,
    ops::{Index, IndexMut},
    path::Display,
};
use wasm_encoder::{CodeSection, ExportKind, ExportSection, FunctionSection, Module, TypeSection};

use crate::{
    ast::function_definition,
    ir::{
        analyzer::{BindedControlFlowGraph, BindedScc, ControlFlowGraph, IsAnalyzer},
        editor::Analyzer,
        statement::IRStatement,
        FunctionDefinition,
    },
};

use self::{
    control_flow::{CFSelector, ControlFlowElement},
    lowering::lower_function_type,
};

mod control_flow;
mod lowering;
// fixme: currently this presumes that we have not folded any if-else or block before
fn fold_loop(
    function_content: &FunctionDefinition,
    scc: &BindedScc,
    current_result: &mut Vec<ControlFlowElement>,
) {
    if let Some(sub_sccs) = scc.top_level_sccs() {
        for sub_scc in sub_sccs
            .into_iter()
            .filter(|sub_scc: &BindedScc<'_>| !sub_scc.is_trivial())
        {
            let sub_scc_start_index = current_result
                .iter()
                .position(|it| {
                    if let &ControlFlowElement::BasicBlock { id: block_id } = it {
                        sub_scc.contains(block_id)
                    } else {
                        false
                    }
                })
                .unwrap();
            let sub_scc_end_index = current_result
                .iter()
                .rposition(|it| {
                    if let &ControlFlowElement::BasicBlock { id: block_id } = it {
                        sub_scc.contains(block_id)
                    } else {
                        false
                    }
                })
                .unwrap();
            let mut new_result = current_result[sub_scc_start_index..=sub_scc_end_index]
                .iter()
                .cloned()
                .collect_vec();
            fold_loop(function_content, &sub_scc, &mut new_result);
            current_result.splice(
                sub_scc_start_index..=sub_scc_end_index,
                [ControlFlowElement::Loop {
                    content: new_result.into_iter().collect(),
                }],
            );
        }
    }
}

fn fold_if_else_once(
    content: &mut ControlFlowElement,
    control_flow_graph: BindedControlFlowGraph,
) -> bool {
    for block_id in 0..control_flow_graph.bind_on.content.len() {
        let predecessors = control_flow_graph.predecessor(block_id);
        if predecessors.len() == 1 {
            let predecessor_block_id = predecessors[0];
            let predecessor_last_instruction = control_flow_graph.bind_on[predecessor_block_id]
                .content
                .last();
            if !matches!(predecessor_last_instruction, Some(IRStatement::Branch(_))) {
                continue;
            }
            let predecessor_selector = content.find_node(predecessor_block_id).unwrap();
            let block_selector = content.find_node(block_id).unwrap();
            let if_element_selector = if predecessor_selector.is_if_condition() {
                // `predecessor_element` is already an if condition
                // in such cases, it's possible that:
                // - the block is already folded into either of branch the `predecessor_element` is in
                //   in such case we don't need to do anything
                // - the block is not folded into any of branch the `predecessor_element` is in
                //   in such case we just fold the block into the `if` element which `predecessor_element` is in
                let mut if_element_predecessor_in_selector = predecessor_selector.clone();
                if_element_predecessor_in_selector.pop_back();
                if if_element_predecessor_in_selector.is_parent_of(&block_selector) {
                    // already folded
                    continue;
                }
                if_element_predecessor_in_selector
            } else {
                // need to promote `predecessor_element` into an if element's condition
                content.replace(
                    &predecessor_selector,
                    ControlFlowElement::If {
                        condition: Box::new(ControlFlowElement::BasicBlock {
                            id: predecessor_block_id,
                        }),
                        on_success: Vec::new(),
                        on_failure: Vec::new(),
                    },
                );
                predecessor_selector.clone()
            };
            let to_move_selectors = collect_to_move(content, &block_selector, &control_flow_graph);
            let to_move_items = to_move_selectors
                .iter()
                .map(|it| content[it].clone())
                .collect_vec();
            let predecessor_element = &mut content[&if_element_selector];
            let predecessor_node_id = predecessor_element.first_basic_block_id();
            let move_to = if control_flow_graph.branch_direction(predecessor_node_id, block_id) {
                if let ControlFlowElement::If { on_success, .. } = predecessor_element {
                    on_success
                } else {
                    unreachable!()
                }
            } else {
                if let ControlFlowElement::If { on_failure, .. } = predecessor_element {
                    on_failure
                } else {
                    unreachable!()
                }
            };
            move_to.extend(to_move_items);
            for to_move_selector in to_move_selectors.iter().rev() {
                content.remove(to_move_selector);
            }
            return true;
        }
    }
    false
}

fn collect_to_move(
    root_element: &ControlFlowElement,
    first_to_move_node_selector: &CFSelector,
    control_flow_graph: &BindedControlFlowGraph<'_, '_>,
) -> Vec<CFSelector> {
    let first_to_move_element = &root_element[first_to_move_node_selector];
    let first_to_move_element_first_bb_id = first_to_move_element.first_basic_block_id();
    let move_to_if_condition_bb_id =
        control_flow_graph.predecessor(first_to_move_element_first_bb_id)[0];
    let mut to_move = vec![first_to_move_node_selector.clone()];
    let mut next = root_element.next_element_sibling(&first_to_move_node_selector);
    while let Some(current_element_selector) = next {
        let current_node_id = root_element[&current_element_selector].first_basic_block_id();
        if control_flow_graph.is_dominated_by(current_node_id, first_to_move_element_first_bb_id)
            && control_flow_graph.is_in_same_branch_side(
                move_to_if_condition_bb_id,
                first_to_move_element_first_bb_id,
                current_node_id,
            )
        {
            to_move.push(current_element_selector.clone());
        } else {
            break;
        }
        next = root_element.next_element_sibling(&current_element_selector);
    }
    to_move
}

fn fold_if_else(function_definition: &FunctionDefinition, content: &mut ControlFlowElement) {
    loop {
        let cfg = ControlFlowGraph::new();
        let control_flow_graph = cfg.bind(function_definition);
        if fold_if_else_once(content, control_flow_graph) {
            break;
        }
    }
}

fn fold(function_definition: &FunctionDefinition) -> Vec<ControlFlowElement> {
    let analyzer = Analyzer::new();
    let binded = analyzer.bind(&function_definition);
    let control_flow_graph = binded.control_flow_graph();
    let current_result = (0..(function_definition.content.len()))
        .map(ControlFlowElement::new_node)
        .collect_vec();

    let mut content = ControlFlowElement::new_block(current_result);
    let control_flow_graph = binded.control_flow_graph();
    let root_scc = control_flow_graph.top_level_scc();
    fold_loop(function_definition, &root_scc, content.unwrap_content_mut());
    fold_if_else(function_definition, &mut content);
    mem::take(content.unwrap_content_mut())
}

fn generate_function(
    result: (
        &mut TypeSection,
        &mut FunctionSection,
        &mut ExportSection,
        &mut CodeSection,
    ),
    function_definition: &FunctionDefinition,
    control_flow: &[ControlFlowElement],
) {
    let function_index = result.0.len();
    let (param_type, return_type) = lower_function_type(&function_definition.header);
    result.0.function(param_type, return_type);
    result.1.function(function_index);
    result.2.export(
        &function_definition.header.name,
        ExportKind::Func,
        function_index,
    );
}

#[cfg(test)]
mod tests {
    use std::{assert_matches::assert_matches, fs, str::FromStr};

    use analyzer::Analyzer;
    use wasm_encoder::{TypeSection, ValType};

    use crate::{
        ir::{
            self,
            analyzer::{self, IsAnalyzer},
            editor::Editor,
            function::{basic_block::BasicBlock, test_util::*},
            optimize::pass::{FixIrreducible, IsPass, TopologicalSort},
            statement::Ret,
        },
        utility::data_type,
    };

    use super::*;

    #[test]
    fn test_fold_if_else_once() {
        let function_definition = FunctionDefinition {
            header: ir::FunctionHeader {
                name: "f".to_string(),
                parameters: Vec::new(),
                return_type: data_type::Type::None,
            },
            content: vec![
                jump_block(0, 1),
                branch_block(1, 2, 3),
                jump_block(2, 4),
                jump_block(3, 4),
                ret_block(4),
            ],
        };
        let mut editor = Editor::new(function_definition);
        let pass = FixIrreducible;
        pass.run(&mut editor);
        let pass = TopologicalSort;
        pass.run(&mut editor);
        let function_definition = editor.content;

        let analyzer = Analyzer::new();
        let binded = analyzer.bind(&function_definition);
        let control_flow_graph = binded.control_flow_graph();
        let current_result = (0..(function_definition.content.len()))
            .map(ControlFlowElement::new_node)
            .collect_vec();

        let mut content = ControlFlowElement::new_block(current_result);
        fold_if_else_once(&mut content, control_flow_graph);
        assert_matches!(
            content[&CFSelector::from_str("1").unwrap()],
            ControlFlowElement::If { .. }
        );
        assert_matches!(
            content[&CFSelector::from_str("1/success/0").unwrap()],
            ControlFlowElement::BasicBlock { id: 2 }
        );
        assert_eq!(
            content.get(&CFSelector::from_str("1/failure/0").unwrap()),
            None
        );
        let control_flow_graph = binded.control_flow_graph();
        fold_if_else_once(&mut content, control_flow_graph);
        assert_eq!(
            content[&CFSelector::from_str("1/failure/0").unwrap()],
            ControlFlowElement::BasicBlock { id: 3 }
        );
        assert_eq!(
            content[&CFSelector::from_str("2").unwrap()],
            ControlFlowElement::BasicBlock { id: 4 }
        );

        let function_definition = FunctionDefinition {
            header: ir::FunctionHeader {
                name: "f".to_string(),
                parameters: Vec::new(),
                return_type: data_type::Type::None,
            },
            content: vec![
                jump_block(0, 1),
                branch_block(1, 2, 4),
                jump_block(2, 3),
                jump_block(4, 5),
                jump_block(3, 6),
                jump_block(5, 6),
                jump_block(6, 7),
                ret_block(7),
            ],
        };
        let mut editor = Editor::new(function_definition);
        let pass = FixIrreducible;
        pass.run(&mut editor);
        let pass = TopologicalSort;
        pass.run(&mut editor);
        let function_definition = editor.content;

        let analyzer = Analyzer::new();
        let binded = analyzer.bind(&function_definition);
        let control_flow_graph = binded.control_flow_graph();
        let current_result = (0..(function_definition.content.len()))
            .map(ControlFlowElement::new_node)
            .collect_vec();

        let mut content = ControlFlowElement::new_block(current_result);
        fold_if_else_once(&mut content, control_flow_graph);
        assert_matches!(
            content[&CFSelector::from_str("1").unwrap()],
            ControlFlowElement::If { .. }
        );
        assert_matches!(
            content[&CFSelector::from_str("1/success/0").unwrap()],
            ControlFlowElement::BasicBlock { id: 2 }
        );
        assert_matches!(
            content[&CFSelector::from_str("1/success/1").unwrap()],
            ControlFlowElement::BasicBlock { id: 3 }
        );
        assert_eq!(
            content.get(&CFSelector::from_str("1/failure/0").unwrap()),
            None
        );
        let control_flow_graph = binded.control_flow_graph();
        fold_if_else_once(&mut content, control_flow_graph);
        assert_matches!(
            content[&CFSelector::from_str("1/failure/0").unwrap()],
            ControlFlowElement::BasicBlock { id: 4 }
        );
        assert_matches!(
            content[&CFSelector::from_str("1/failure/1").unwrap()],
            ControlFlowElement::BasicBlock { id: 5 }
        );

        let function_definition = FunctionDefinition {
            header: ir::FunctionHeader {
                name: "f".to_string(),
                parameters: Vec::new(),
                return_type: data_type::Type::None,
            },
            content: vec![
                branch_block(0, 1, 5),
                jump_block(1, 2),
                jump_block(2, 4),
                branch_block(4, 3, 7),
                jump_block(3, 2),
                ret_block(7),
                jump_block(5, 6),
                jump_block(6, 7),
            ],
        };
        let mut editor = Editor::new(function_definition);
        let pass = FixIrreducible;
        pass.run(&mut editor);
        let pass = TopologicalSort;
        pass.run(&mut editor);
        let function_definition = editor.content;
        let analyzer = Analyzer::new();
        let binded = analyzer.bind(&function_definition);
        let control_flow_graph = binded.control_flow_graph();
        let mut content = ControlFlowElement::new_block(vec![
            ControlFlowElement::new_node(0),
            ControlFlowElement::new_node(1),
            ControlFlowElement::Loop {
                content: vec![
                    ControlFlowElement::new_node(2),
                    ControlFlowElement::new_node(4),
                    ControlFlowElement::new_node(3),
                ],
            },
            ControlFlowElement::new_node(5),
            ControlFlowElement::new_node(6),
            ControlFlowElement::new_node(7),
        ]);
        fold_if_else_once(&mut content, control_flow_graph);
        let control_flow_graph = binded.control_flow_graph();
        fold_if_else_once(&mut content, control_flow_graph);
        let control_flow_graph = binded.control_flow_graph();
        fold_if_else_once(&mut content, control_flow_graph);
        assert_matches!(
            &content[&CFSelector::from_str("0").unwrap()],
            ControlFlowElement::If { .. }
        );
        assert_matches!(
            &content[&CFSelector::from_str("0/if_condition").unwrap()],
            ControlFlowElement::BasicBlock { id: 0 }
        );
        assert_matches!(
            &content[&CFSelector::from_str("0/success/0").unwrap()],
            ControlFlowElement::BasicBlock { id: 1 }
        );
        assert_matches!(
            &content[&CFSelector::from_str("0/success/1").unwrap()],
            ControlFlowElement::Loop { .. }
        );
        assert_matches!(
            &content[&CFSelector::from_str("0/success/1/0").unwrap()],
            ControlFlowElement::BasicBlock { id: 2 }
        );
        assert_matches!(
            &content[&CFSelector::from_str("0/success/1/1").unwrap()],
            ControlFlowElement::If { .. }
        );
        assert_matches!(
            &content[&CFSelector::from_str("0/success/1/1/if_condition").unwrap()],
            ControlFlowElement::BasicBlock { id: 3 }
        );
        assert_matches!(
            &content[&CFSelector::from_str("0/success/1/1/success/0").unwrap()],
            ControlFlowElement::BasicBlock { id: 4 }
        );
    }

    #[test]
    fn test_loop() {
        let function_definition = FunctionDefinition {
            header: ir::FunctionHeader {
                name: "f".to_string(),
                parameters: Vec::new(),
                return_type: data_type::Type::None,
            },
            content: vec![
                BasicBlock {
                    name: Some("bb0".to_string()),
                    content: vec![jump("bb1")],
                },
                BasicBlock {
                    name: Some("bb1".to_string()),
                    content: vec![branch("bb2", "bb8")],
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
                    content: vec![jump("bb5")],
                },
                BasicBlock {
                    name: Some("bb5".to_string()),
                    content: vec![branch("bb6", "bb7")],
                },
                BasicBlock {
                    name: Some("bb6".to_string()),
                    content: vec![jump("bb3")],
                },
                BasicBlock {
                    name: Some("bb7".to_string()),
                    content: vec![branch("bb2", "bb15")],
                },
                BasicBlock {
                    name: Some("bb8".to_string()),
                    content: vec![branch("bb9", "bb10")],
                },
                BasicBlock {
                    name: Some("bb9".to_string()),
                    content: vec![jump("bb11")],
                },
                BasicBlock {
                    name: Some("bb11".to_string()),
                    content: vec![jump("bb12")],
                },
                BasicBlock {
                    name: Some("bb12".to_string()),
                    content: vec![jump("bb13")],
                },
                BasicBlock {
                    name: Some("bb10".to_string()),
                    content: vec![jump("bb12")],
                },
                BasicBlock {
                    name: Some("bb13".to_string()),
                    content: vec![jump("bb14")],
                },
                BasicBlock {
                    name: Some("bb14".to_string()),
                    content: vec![branch("bb12", "bb15")],
                },
                BasicBlock {
                    name: Some("bb15".to_string()),
                    content: vec![jump("bb16")],
                },
                BasicBlock {
                    name: Some("bb16".to_string()),
                    content: vec![Ret { value: None }.into()],
                },
            ],
        };
        let mut editor = Editor::new(function_definition);
        let pass = FixIrreducible;
        pass.run(&mut editor);
        let pass = TopologicalSort;
        pass.run(&mut editor);
        let function_definition = editor.content;

        let analyzer = Analyzer::new();
        let binded = analyzer.bind(&function_definition);
        let control_flow_graph = binded.control_flow_graph();
        let scc = control_flow_graph.top_level_scc();

        let mut current_result = (0..(function_definition.content.len()))
            .map(ControlFlowElement::new_node)
            .collect_vec();
        fold_loop(&function_definition, &scc, &mut current_result);
        dbg!(current_result);
    }

    #[test]
    fn test_fold_all() {
        let function_definition = FunctionDefinition {
            header: ir::FunctionHeader {
                name: "f".to_string(),
                parameters: Vec::new(),
                return_type: data_type::Type::None,
            },
            content: vec![
                jump_block(0, 1),
                jump_block(1, 2),
                jump_block(2, 3),
                branch_block(3, 4, 1),
                branch_block(4, 1, 5),
                ret_block(5),
            ],
        };
        let result = fold(&function_definition);
        dbg!(result);
    }
}
