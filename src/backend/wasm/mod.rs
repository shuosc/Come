use itertools::Itertools;

use crate::ir::{
    analyzer::{self, BindedControlFlowGraph, SccContent},
    statement::IRStatement,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlFlowContent {
    Block(Vec<ControlFlowContent>),
    If(Vec<ControlFlowContent>, Vec<ControlFlowContent>),
    Loop(Vec<ControlFlowContent>),
    Node(usize),
}

impl ControlFlowContent {
    pub fn new_block(content: Vec<ControlFlowContent>) -> Self {
        Self::Block(content)
    }

    pub fn new_if(taken: Vec<ControlFlowContent>, untaken: Vec<ControlFlowContent>) -> Self {
        Self::If(taken, untaken)
    }

    pub fn new_loop(content: Vec<ControlFlowContent>) -> Self {
        Self::Loop(content)
    }

    pub fn new_node(node: usize) -> Self {
        Self::Node(node)
    }

    pub fn first_node(&self) -> usize {
        match self {
            ControlFlowContent::Block(content) | ControlFlowContent::Loop(content) => {
                content.first().unwrap().first_node()
            }
            ControlFlowContent::If(taken, _untaken) => taken.first().unwrap().first_node(),
            ControlFlowContent::Node(n) => *n,
        }
    }

    pub fn get<T: Into<usize> + Clone>(&self, index: &[T]) -> Option<&Self> {
        let current_index = index.get(0)?.clone();
        let current_index = current_index.into();
        let current = match self {
            ControlFlowContent::Block(content) | ControlFlowContent::Loop(content) => {
                content.get(current_index)
            }
            ControlFlowContent::If(taken, untaken) => taken
                .get(current_index)
                .or_else(|| untaken.get(current_index - taken.len())),
            ControlFlowContent::Node(_) => {
                if current_index == 0 {
                    Some(self)
                } else {
                    return None;
                }
            }
        };
        let rest_index = &index[1..];
        if rest_index.is_empty() {
            current
        } else {
            current?.get(rest_index)
        }
    }

    pub fn get_mut<T: Into<usize> + Clone>(&mut self, index: &[T]) -> Option<&mut Self> {
        let current_index = index.get(0)?.clone();
        let current_index = current_index.into();
        let current = match self {
            ControlFlowContent::Block(content) | ControlFlowContent::Loop(content) => {
                content.get_mut(current_index)
            }
            ControlFlowContent::If(taken, untaken) => {
                if current_index < taken.len() {
                    taken.get_mut(current_index)
                } else {
                    untaken.get_mut(current_index - taken.len())
                }
            }
            ControlFlowContent::Node(_) => {
                if current_index == 0 {
                    Some(self)
                } else {
                    return None;
                }
            }
        };
        let rest_index = &index[1..];
        if rest_index.is_empty() {
            current
        } else {
            current?.get_mut(rest_index)
        }
    }

    pub fn contains(&self, node: usize) -> bool {
        match self {
            ControlFlowContent::Block(content) | ControlFlowContent::Loop(content) => {
                content.iter().any(|it| it.contains(node))
            }
            ControlFlowContent::If(taken, untaken) => taken
                .iter()
                .chain(untaken.iter())
                .any(|it| it.contains(node)),
            ControlFlowContent::Node(n) => *n == node,
        }
    }

    pub fn remove<T: Into<usize> + Clone>(&mut self, index: &[T]) -> Option<Self> {
        if index.is_empty() {
            None
        } else if index.len() == 1 {
            let index = index[0].clone().into();
            match self {
                ControlFlowContent::Block(content) | ControlFlowContent::Loop(content) => {
                    Some(content.remove(index))
                }
                ControlFlowContent::If(taken, untaken) => {
                    if index < taken.len() {
                        Some(taken.remove(index))
                    } else {
                        Some(untaken.remove(index))
                    }
                }
                ControlFlowContent::Node(n) => {
                    panic!("unable to remove the {index}th element from node {n}")
                }
            }
        } else {
            self.get_mut(&[index[0].clone()])
                .unwrap()
                .remove(&index[1..])
        }
    }

    pub fn position(&self, item: &Self) -> Option<Vec<usize>> {
        match self {
            ControlFlowContent::Block(content) | ControlFlowContent::Loop(content) => {
                for (i, subblock) in content.iter().enumerate() {
                    let mut potential_result = vec![i];
                    if subblock == item {
                        return Some(potential_result);
                    } else if let Some(result) = subblock.position(item) {
                        potential_result.extend_from_slice(&result);
                        return Some(potential_result);
                    }
                }
            }
            ControlFlowContent::If(taken, untaken) => {
                for (i, subblock) in taken.iter().chain(untaken.iter()).enumerate() {
                    let mut potential_result = vec![i];
                    if subblock == item {
                        return Some(potential_result);
                    } else if let Some(result) = subblock.position(item) {
                        potential_result.extend_from_slice(&result);
                        return Some(potential_result);
                    }
                }
            }
            ControlFlowContent::Node(_) => {
                if self == item {
                    return Some(vec![0]);
                }
            }
        }
        None
    }

    pub fn nodes(&self) -> ControlFlowNodesIter {
        ControlFlowNodesIter::new(self)
    }
}

pub struct ControlFlowNodesIter<'a> {
    bind_on: &'a ControlFlowContent,
    pub current_index: Vec<usize>,
}

impl<'a> ControlFlowNodesIter<'a> {
    pub fn new(bind_on: &'a ControlFlowContent) -> Self {
        Self {
            bind_on,
            current_index: vec![0],
        }
    }

    pub fn from_index(bind_on: &'a ControlFlowContent, index: &[usize]) -> Self {
        Self {
            bind_on,
            current_index: index.to_vec(),
        }
    }
}

impl Iterator for ControlFlowNodesIter<'_> {
    type Item = (Vec<usize>, usize);

    fn next(&mut self) -> Option<Self::Item> {
        match self.bind_on.get(&self.current_index) {
            Some(ControlFlowContent::Block(_))
            | Some(ControlFlowContent::Loop(_))
            | Some(ControlFlowContent::If(_, _)) => {
                self.current_index.push(0);
                self.next()
            }
            Some(ControlFlowContent::Node(n)) => {
                let result = self.current_index.clone();
                *self.current_index.last_mut().unwrap() += 1;
                Some((result, *n))
            }
            None => {
                if self.current_index.len() != 1 {
                    self.current_index.pop();
                    *self.current_index.last_mut().unwrap() += 1;
                    self.next()
                } else {
                    None
                }
            }
        }
    }
}

fn fold_loop(current: &mut ControlFlowContent, loop_item: &analyzer::Scc) {
    let (to_remove_indexes, to_remove_items): (Vec<_>, Vec<_>) = current
        .nodes()
        .filter(|(_, n)| loop_item.is_node_in((*n).into()))
        .unzip();
    for to_remove_index in to_remove_indexes[1..].iter().rev() {
        current.remove(to_remove_index);
    }
    let first = current.get_mut(&to_remove_indexes[0]).unwrap();
    let new_loop_item = ControlFlowContent::Loop(
        to_remove_items
            .into_iter()
            .map(ControlFlowContent::Node)
            .collect(),
    );
    *first = new_loop_item;
    for content in &loop_item.content {
        if let SccContent::SubScc(subloop) = content {
            fold_loop(first, subloop);
        }
    }
}

fn fold_if_else_once(current: &mut ControlFlowContent, graph: &BindedControlFlowGraph) -> bool {
    // A node is foldable if its only successor is an branch block
    // So for each branch block:
    //   - if the "next" block has only one successor, nest it and nodes dominated by it in an if
    //   - if (the "next" block after the new nested if)'s only successor is also the block, nest it in the else part
    let (node_indexes, nodes): (Vec<_>, Vec<_>) = current.nodes().unzip();
    let mut considering_node_index = 0;
    let mut folded = false;
    while considering_node_index < nodes.len() - 1 {
        let node = nodes[considering_node_index];
        let mut next_node_index = node_indexes[considering_node_index].clone();
        *next_node_index.last_mut().unwrap() += 1;
        if matches!(
            current.get(&next_node_index),
            Some(ControlFlowContent::If(_, _))
        ) {
            // already nested, just consider next
            considering_node_index += 1;
            continue;
        }
        let block = &graph.bind_on.content[node];
        let last_statement = block.content.last().unwrap();
        if let IRStatement::Branch(_) = last_statement {
            let next_node = nodes[considering_node_index + 1];
            if graph.not_dominate_successors(next_node).len() == 1 {
                let nodes_dominated_by_next_node = graph.dominates(next_node);
                let mut to_nest = Vec::new();
                let mut next_to_nest_index = node_indexes[considering_node_index + 1].clone();
                // the next node is deep nested in loops, so we need to fold all structure the node is in
                while next_to_nest_index.len() > node_indexes[considering_node_index].len() {
                    next_to_nest_index.pop();
                }
                while let Some(next_to_nest) = current.get(&next_to_nest_index) && nodes_dominated_by_next_node.contains(&next_to_nest.first_node()) {
                    to_nest.push(next_to_nest_index.clone());
                    *next_to_nest_index.last_mut().unwrap() += 1;
                }
                let initial_considering_node_index = considering_node_index;
                considering_node_index += to_nest
                    .iter()
                    .map(|it| current.get(it).unwrap().nodes().count())
                    .sum::<usize>();
                let (to_replace, to_remove) = to_nest.split_first().unwrap();
                let removed = to_remove
                    .iter()
                    .map(|it| current.remove(it).unwrap())
                    .collect_vec();
                // nest else part
                let node_after_nest = nodes[considering_node_index];
                let node_after_nest_successors = graph.not_dominate_successors(node_after_nest);
                let untaken_content = if node_after_nest_successors.len() == 1 {
                    let nodes_dominated_by_node_after_nest = graph.dominates(node_after_nest);
                    let mut to_nest = Vec::new();
                    let mut next_to_nest_index = node_indexes[considering_node_index].clone();
                    // the next node is deep nested in loops, so we need to fold all structure the node is in
                    while next_to_nest_index.len()
                        > node_indexes[initial_considering_node_index].len()
                    {
                        next_to_nest_index.pop();
                    }
                    while let Some(next_to_nest) = current.get(&next_to_nest_index) && nodes_dominated_by_node_after_nest.contains(&next_to_nest.first_node()) {
                        to_nest.push(next_to_nest_index.clone());
                        *next_to_nest_index.last_mut().unwrap() += 1;
                    }
                    considering_node_index += to_nest.len();
                    to_nest
                        .iter()
                        .map(|it| current.remove(it).unwrap())
                        .collect_vec()
                } else {
                    Vec::new()
                };
                let replaced_node = current.get_mut(to_replace).unwrap();
                let mut taken_content = vec![replaced_node.clone()];
                taken_content.extend_from_slice(&removed);
                let new_if_node = ControlFlowContent::If(taken_content, untaken_content);
                *replaced_node = new_if_node;
                folded = true;
            } else {
                considering_node_index += 1;
            }
        } else {
            considering_node_index += 1;
        }
    }
    folded
}

fn fold_if_else(current: &mut ControlFlowContent, graph: &BindedControlFlowGraph) {
    let mut folded = true;
    while folded {
        folded = fold_if_else_once(current, graph);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ir::{
            self,
            analyzer::{ControlFlowGraph, IsAnalyzer},
            function::{basic_block::BasicBlock, test_util::*},
            statement::Ret,
            FunctionDefinition,
        },
        utility::data_type,
    };
    #[test]
    fn control_flow_content_get() {
        let content = ControlFlowContent::new_block(vec![
            ControlFlowContent::new_node(0),
            ControlFlowContent::new_if(
                vec![
                    ControlFlowContent::new_node(1),
                    ControlFlowContent::new_node(2),
                ],
                vec![ControlFlowContent::new_loop(vec![
                    ControlFlowContent::new_node(3),
                    ControlFlowContent::new_node(4),
                ])],
            ),
        ]);
        assert_eq!(
            content.get(&[0usize]),
            Some(&ControlFlowContent::new_node(0))
        );
        assert_eq!(
            content.get(&[1usize, 0]),
            Some(&ControlFlowContent::new_node(1))
        );
        assert_eq!(
            content.get(&[1usize, 2]),
            Some(&ControlFlowContent::new_loop(vec![
                ControlFlowContent::new_node(3),
                ControlFlowContent::new_node(4),
            ]))
        );
        assert_eq!(
            content.get(&[1usize, 2, 0]),
            Some(&ControlFlowContent::new_node(3))
        );
        assert_eq!(content.get(&[2usize, 0, 2]), None);
        assert_eq!(content.get(&[3usize]), None);
    }

    #[test]
    fn control_flow_content_position() {
        let content = ControlFlowContent::new_block(vec![
            ControlFlowContent::new_node(0),
            ControlFlowContent::new_if(
                vec![
                    ControlFlowContent::new_node(1),
                    ControlFlowContent::new_node(2),
                ],
                vec![ControlFlowContent::new_loop(vec![
                    ControlFlowContent::new_node(3),
                    ControlFlowContent::new_node(4),
                ])],
            ),
        ]);
        assert_eq!(
            content.position(&ControlFlowContent::new_node(0)),
            Some(vec![0])
        );
        assert_eq!(
            content.position(&ControlFlowContent::new_node(1)),
            Some(vec![1, 0])
        );
        assert_eq!(
            content.position(&ControlFlowContent::new_loop(vec![
                ControlFlowContent::new_node(3),
                ControlFlowContent::new_node(4),
            ])),
            Some(vec![1, 2])
        );
        assert_eq!(
            content.position(&ControlFlowContent::new_node(3)),
            Some(vec![1, 2, 0])
        );
        assert_eq!(content.position(&ControlFlowContent::new_node(5)), None);
    }

    #[test]
    fn control_flow_nodes() {
        let content = ControlFlowContent::new_block(vec![
            ControlFlowContent::new_node(0),
            ControlFlowContent::new_if(
                vec![
                    ControlFlowContent::new_node(1),
                    ControlFlowContent::new_node(2),
                ],
                vec![ControlFlowContent::new_loop(vec![
                    ControlFlowContent::new_node(3),
                    ControlFlowContent::new_node(4),
                ])],
            ),
            ControlFlowContent::new_node(5),
        ]);
        let mut iter = content.nodes();
        assert_eq!(iter.next(), Some((vec![0], 0)));
        assert_eq!(iter.next(), Some((vec![1, 0], 1)));
        assert_eq!(iter.next(), Some((vec![1, 1], 2)));
        assert_eq!(iter.next(), Some((vec![1, 2, 0], 3)));
        assert_eq!(iter.next(), Some((vec![1, 2, 1], 4)));
        assert_eq!(iter.next(), Some((vec![2], 5)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_fold_loop() {
        let mut content = ControlFlowContent::new_block(vec![
            ControlFlowContent::new_node(0),
            ControlFlowContent::new_node(1),
            ControlFlowContent::new_node(2),
            ControlFlowContent::new_node(3),
            ControlFlowContent::new_node(4),
        ]);
        let loop_item = analyzer::Scc {
            entries: vec![1],
            content: vec![
                analyzer::SccContent::Node(1),
                analyzer::SccContent::Node(2),
                analyzer::SccContent::Node(3),
            ],
        };
        fold_loop(&mut content, &loop_item);
        assert_eq!(
            content,
            ControlFlowContent::new_block(vec![
                ControlFlowContent::new_node(0),
                ControlFlowContent::new_loop(vec![
                    ControlFlowContent::new_node(1),
                    ControlFlowContent::new_node(2),
                    ControlFlowContent::new_node(3),
                ]),
                ControlFlowContent::new_node(4),
            ])
        );

        let mut content = ControlFlowContent::new_block(vec![
            ControlFlowContent::new_node(0),
            ControlFlowContent::new_node(1),
            ControlFlowContent::new_node(2),
            ControlFlowContent::new_node(3),
            ControlFlowContent::new_node(4),
            ControlFlowContent::new_node(5),
            ControlFlowContent::new_node(6),
        ]);
        let loop_item = analyzer::Scc {
            entries: vec![1],
            content: vec![
                analyzer::SccContent::Node(1),
                analyzer::SccContent::Node(2),
                analyzer::SccContent::SubScc(Box::new(analyzer::Scc {
                    entries: vec![3],
                    content: vec![
                        analyzer::SccContent::Node(3),
                        analyzer::SccContent::Node(4),
                        analyzer::SccContent::Node(5),
                    ],
                })),
            ],
        };
        fold_loop(&mut content, &loop_item);
        assert_eq!(
            content,
            ControlFlowContent::new_block(vec![
                ControlFlowContent::new_node(0),
                ControlFlowContent::new_loop(vec![
                    ControlFlowContent::new_node(1),
                    ControlFlowContent::new_node(2),
                    ControlFlowContent::new_loop(vec![
                        ControlFlowContent::new_node(3),
                        ControlFlowContent::new_node(4),
                        ControlFlowContent::new_node(5),
                    ]),
                ]),
                ControlFlowContent::new_node(6),
            ])
        );
    }

    #[test]
    fn test_fold_if_else() {
        let function_definition = FunctionDefinition {
            header: ir::FunctionHeader {
                name: "f".to_string(),
                parameters: Vec::new(),
                return_type: data_type::Type::None,
            },
            content: vec![
                BasicBlock {
                    name: Some("bb0".to_string()),
                    content: vec![branch("bb1", "bb2")],
                },
                BasicBlock {
                    name: Some("bb1".to_string()),
                    content: vec![jump("bb3")],
                },
                BasicBlock {
                    name: Some("bb3".to_string()),
                    content: vec![branch("bb4", "bb5")],
                },
                BasicBlock {
                    name: Some("bb4".to_string()),
                    content: vec![jump("bb6")],
                },
                BasicBlock {
                    name: Some("bb5".to_string()),
                    content: vec![jump("bb6")],
                },
                BasicBlock {
                    name: Some("bb6".to_string()),
                    content: vec![branch("bb1", "bb7")],
                },
                BasicBlock {
                    name: Some("bb2".to_string()),
                    content: vec![jump("bb7")],
                },
                BasicBlock {
                    name: Some("bb7".to_string()),
                    content: vec![Ret { value: None }.into()],
                },
            ],
        };
        let control_flow_graph = ControlFlowGraph::new();
        let binded = control_flow_graph.bind(&function_definition);
        let mut content = ControlFlowContent::new_block(vec![
            ControlFlowContent::new_node(0),
            ControlFlowContent::new_loop(vec![
                ControlFlowContent::new_node(1),
                ControlFlowContent::new_node(2),
                ControlFlowContent::new_node(3),
                ControlFlowContent::new_node(4),
                ControlFlowContent::new_node(5),
            ]),
            ControlFlowContent::new_node(6),
            ControlFlowContent::new_node(7),
        ]);
        fold_if_else(&mut content, &binded);
        assert!(matches!(
            content.get(&[1usize]),
            Some(ControlFlowContent::If(_, _))
        ));
        assert!(matches!(
            content.get(&[1usize, 0, 2]),
            Some(ControlFlowContent::If(_, _))
        ));
    }
}
