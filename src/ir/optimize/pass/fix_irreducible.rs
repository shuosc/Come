use super::IsPass;
use crate::{
    ir::{
        statement::{
            branch::BranchType, phi::PhiSource, BinaryCalculate, Branch, IRStatement, Jump, Phi,
        },
        RegisterName,
    },
    utility::data_type,
};

use itertools::Itertools;
use petgraph::prelude::*;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct FixIrreducible;

pub enum GoToCondition {
    Register(RegisterName),
    Always,
}

// If `condition` is true, the connections from `from` should be redirected
struct ShouldGoTo {
    from: usize,
    condition: GoToCondition,
    dispatcher: usize,
    on_success_branch: bool,
}

fn fold_entries_once(
    loop_name: &str,
    header: &mut Option<NodeIndex<usize>>,
    generated: &mut Vec<NodeIndex<usize>>,
    entries: &mut Vec<(NodeIndex<usize>, Vec<NodeIndex<usize>>)>,
    editor: &mut crate::ir::editor::Editor,
) -> Vec<ShouldGoTo> {
    let last = generated.last();
    if entries.len() >= 2 {
        let new_node_name = format!(
            "{}_dispatcher_at_{}",
            loop_name,
            editor.content.content.len()
        );
        let new_node = editor.create_basic_block(new_node_name.clone());
        if let Some(last) = last {
            let last_bb_size = editor.content[last.index()].content.len();
            let mut last_bb_last_statement =
                editor.content[last.index()].content.last().unwrap().clone();
            editor.remove_statement((last.index(), last_bb_size - 1));
            let IRStatement::Branch(branch) = &mut last_bb_last_statement else { unreachable!() };
            branch.failure_label = new_node_name.clone();
            editor.push_back_statement(last.index(), last_bb_last_statement);
        }
        let (branch_to, blocks_into_branch_to) = entries.pop().unwrap();
        let branch_to_bb_name = editor.content[branch_to.index()].name.clone().unwrap();
        let operand_name = format!("{new_node_name}_var");
        let new_branch_statement = Branch {
            branch_type: BranchType::NE,
            operand1: RegisterName(operand_name).into(),
            operand2: 0.into(),
            success_label: branch_to_bb_name,
            failure_label: String::new(),
        };
        editor.push_back_statement(new_node, new_branch_statement);
        let header = header.get_or_insert(new_node.into());
        generated.push(new_node.into());
        fix_branch_into_source(
            loop_name,
            editor,
            header.index(),
            branch_to.index(),
            &blocks_into_branch_to,
            new_node,
        )
    } else {
        let last = last.unwrap();
        let last_bb_size = editor.content[last.index()].content.len();
        let mut last_bb_last_statement =
            editor.content[last.index()].content.last().unwrap().clone();
        editor.remove_statement((last.index(), last_bb_size - 1));
        let IRStatement::Branch(branch) = &mut last_bb_last_statement else { unreachable!() };
        let (branch_to, blocks_into_branch_to) = entries.pop().unwrap();
        let branch_to_bb_name = editor.content[branch_to.index()].name.clone().unwrap();
        branch.failure_label = branch_to_bb_name;
        editor.push_back_statement(last.index(), last_bb_last_statement);
        let header = header.as_mut().unwrap();
        let mut result = fix_branch_into_source(
            loop_name,
            editor,
            header.index(),
            branch_to.index(),
            &blocks_into_branch_to,
            last.index(),
        );
        for r in result.iter_mut() {
            r.on_success_branch = false;
        }
        result
    }
}

fn fix_branch_into_source(
    loop_name: &str,
    editor: &mut crate::ir::editor::Editor,
    header: usize,
    branch_to: usize,
    from_blocks: &[NodeIndex<usize>],
    dispatcher: usize,
) -> Vec<ShouldGoTo> {
    let header_block_name = format!("{loop_name}_dispatcher_at_{header}");
    let branch_to_block_name = editor.content[branch_to].name.clone().unwrap();
    let mut should_go_tos = Vec::new();
    for from_block in from_blocks {
        let from_block = from_block.index();
        let mut from_block_last_statement =
            editor.content[from_block].content.last().unwrap().clone();
        editor.remove_statement((from_block, editor.content[from_block].content.len() - 1));

        let condition_register: RegisterName =
            RegisterName(format!("{loop_name}_goto_{branch_to}_in_{from_block}"));
        let should_go_to = match &mut from_block_last_statement {
            IRStatement::Branch(branch) => {
                if branch.success_label == header_block_name {
                    let condition_statement = BinaryCalculate {
                        operation: branch
                            .branch_type
                            .corresponding_binary_operation()
                            .inverse()
                            .unwrap(),
                        operand1: branch.operand1.clone(),
                        operand2: branch.operand2.clone(),
                        to: condition_register.clone(),
                        data_type: data_type::Integer {
                            signed: false,
                            width: 1,
                        }
                        .into(),
                    };
                    editor.push_back_statement(from_block, condition_statement);
                    from_block_last_statement = Jump {
                        label: header_block_name.clone(),
                    }
                    .into();
                } else if branch.failure_label == header_block_name {
                    let condition_statement = BinaryCalculate {
                        operation: branch.branch_type.corresponding_binary_operation(),
                        operand1: branch.operand1.clone(),
                        operand2: branch.operand2.clone(),
                        to: condition_register.clone(),
                        data_type: data_type::Integer {
                            signed: false,
                            width: 1,
                        }
                        .into(),
                    };
                    editor.push_back_statement(from_block, condition_statement);
                    from_block_last_statement = Jump {
                        label: header_block_name.clone(),
                    }
                    .into();
                } else if branch.success_label == branch_to_block_name {
                    let condition_statement = BinaryCalculate {
                        operation: branch.branch_type.corresponding_binary_operation(),
                        operand1: branch.operand1.clone(),
                        operand2: branch.operand2.clone(),
                        to: condition_register.clone(),
                        data_type: data_type::Integer {
                            signed: false,
                            width: 1,
                        }
                        .into(),
                    };
                    editor.push_back_statement(from_block, condition_statement);
                    branch.success_label = header_block_name.clone();
                } else {
                    let condition_statement = BinaryCalculate {
                        operation: branch
                            .branch_type
                            .inverse()
                            .corresponding_binary_operation(),
                        operand1: branch.operand1.clone(),
                        operand2: branch.operand2.clone(),
                        to: condition_register.clone(),
                        data_type: data_type::Integer {
                            signed: false,
                            width: 1,
                        }
                        .into(),
                    };
                    editor.push_back_statement(from_block, condition_statement);
                    branch.failure_label = header_block_name.clone();
                }
                ShouldGoTo {
                    from: from_block,
                    condition: GoToCondition::Register(condition_register.clone()),
                    dispatcher,
                    on_success_branch: true,
                }
            }
            IRStatement::Jump(jump) => {
                jump.label = header_block_name.clone();
                ShouldGoTo {
                    from: from_block,
                    condition: GoToCondition::Always,
                    dispatcher,
                    on_success_branch: true,
                }
            }
            _ => unreachable!(),
        };
        editor.push_back_statement(from_block, from_block_last_statement);
        should_go_tos.push(should_go_to);
    }
    should_go_tos
}

// fixme
fn generate_phis(
    loop_name: &str,
    all_should_go_to: &[ShouldGoTo],
    generated: &[NodeIndex<usize>],
    editor: &mut crate::ir::editor::Editor,
) -> Vec<Phi> {
    let sources = all_should_go_to.iter().map(|it| it.from).collect_vec();
    let binding = all_should_go_to.iter().group_by(|it| it.dispatcher);
    let groups = binding
        .into_iter()
        .sorted_by_cached_key(|(it, _)| {
            generated
                .iter()
                .position(|generated_index| generated_index.index() == *it)
        })
        .collect_vec();
    generated
        .iter()
        .zip(groups.into_iter())
        .map(|(generated, (_, should_gotos))| {
            let generated = generated.index();
            let should_go_to = should_gotos
                .into_iter()
                .filter(|it| it.on_success_branch)
                .collect_vec();
            let condition_register: RegisterName =
                RegisterName(format!("{loop_name}_dispatcher_at_{generated}_var"));
            let not_related_sources = sources
                .iter()
                .filter(|&&source| !should_go_to.iter().any(|it| it.from == source))
                .collect_vec();
            let from = should_go_to
                .iter()
                .map(|should_go_to_item| PhiSource {
                    value: match &should_go_to_item.condition {
                        GoToCondition::Register(register) => register.clone().into(),
                        GoToCondition::Always => 1.into(),
                    },
                    block: editor
                        .binded_analyzer()
                        .control_flow_graph()
                        .basic_block_name_by_index(should_go_to_item.from)
                        .to_string(),
                })
                .chain(not_related_sources.into_iter().map(|it| {
                    PhiSource {
                        value: 0.into(),
                        block: editor
                            .binded_analyzer()
                            .control_flow_graph()
                            .basic_block_name_by_index(*it)
                            .to_string(),
                    }
                }))
                .collect();
            Phi {
                to: condition_register,
                data_type: data_type::Integer {
                    signed: false,
                    width: 1,
                }
                .into(),
                from,
            }
        })
        .collect()
}

impl IsPass for FixIrreducible {
    fn run(&self, editor: &mut crate::ir::editor::Editor) {
        loop {
            let mut header = None;
            let binded_analyzer = editor.binded_analyzer();
            let control_flow_graph = binded_analyzer.control_flow_graph();
            let (mut entries, loop_name) = if let Some(irreducible_loop) =
                control_flow_graph.loops().first_irreducible_loop()
            {
                (
                    irreducible_loop.entry_info(control_flow_graph.graph()),
                    irreducible_loop.name(),
                )
            } else {
                break;
            };
            let mut generated = Vec::new();
            let mut should_go_to = Vec::new();
            while !entries.is_empty() {
                should_go_to.append(&mut fold_entries_once(
                    &loop_name,
                    &mut header,
                    &mut generated,
                    &mut entries,
                    editor,
                ));
            }
            let phis = generate_phis(&loop_name, &should_go_to, &generated, editor);
            let header = header.unwrap().index();
            for phi in phis.into_iter() {
                editor.push_front_statement(header, phi);
            }
        }
    }

    fn need(&self) -> Vec<super::Pass> {
        Vec::new()
    }

    fn invalidate(&self) -> Vec<super::Pass> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::ir::{
        self,
        editor::Editor,
        function::{basic_block::BasicBlock, test_util::*},
        statement::{calculate::binary::BinaryOperation, Ret},
        FunctionDefinition,
    };

    use super::*;
    #[test]
    fn simple() {
        let function_definition = FunctionDefinition {
            header: ir::FunctionHeader {
                name: "f".to_string(),
                parameters: Vec::new(),
                return_type: data_type::Type::None,
            },
            content: vec![
                BasicBlock {
                    name: Some("bb0".to_string()),
                    content: vec![branch("bb1", "bb5")],
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
                    content: vec![branch("bb4", "bb6")],
                },
                BasicBlock {
                    name: Some("bb4".to_string()),
                    content: vec![jump("bb1")],
                },
                BasicBlock {
                    name: Some("bb5".to_string()),
                    content: vec![branch("bb4", "bb6")],
                },
                BasicBlock {
                    name: Some("bb6".to_string()),
                    content: vec![Ret { value: None }.into()],
                },
            ],
        };
        let mut editor = Editor::new(function_definition);
        let pass = FixIrreducible;
        pass.run(&mut editor);
        assert_eq!(editor.content.content.len(), 8);
        let analyzer = editor.binded_analyzer();
        let cfg = analyzer.control_flow_graph();
        assert_eq!(cfg.loops().content.len(), 5);
    }

    #[test]
    fn double_dispatcher() {
        let function_definition = FunctionDefinition {
            header: ir::FunctionHeader {
                name: "f".to_string(),
                parameters: Vec::new(),
                return_type: data_type::Type::None,
            },
            content: vec![
                BasicBlock {
                    name: Some("bb0".to_string()),
                    content: vec![branch("bb1", "bb6")],
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
                    content: vec![branch("bb4", "bb5")],
                },
                BasicBlock {
                    name: Some("bb4".to_string()),
                    content: vec![branch("bb2", "bb1")],
                },
                BasicBlock {
                    name: Some("bb5".to_string()),
                    content: vec![Ret { value: None }.into()],
                },
                BasicBlock {
                    name: Some("bb6".to_string()),
                    content: vec![branch("bb4", "bb3")],
                },
            ],
        };
        let mut editor = Editor::new(function_definition);
        let pass = FixIrreducible;
        pass.run(&mut editor);
        assert_eq!(editor.content.content.len(), 9);
        let bb6 = editor
            .content
            .content
            .iter()
            .find(|it| it.name.as_ref().map(|it| it == "bb6").unwrap_or(false))
            .unwrap();
        assert_eq!(bb6.content.len(), 3);
        let register_4_to_6_setting = bb6
            .content
            .iter()
            .filter_map(|it| it.try_as_binary_calculate())
            .find(|it| it.to == RegisterName("_loop_3_4_1_goto_4_in_6".to_string()))
            .unwrap();
        assert_eq!(register_4_to_6_setting.operation, BinaryOperation::Equal);
        assert_eq!(register_4_to_6_setting.operand1, 0.into());
        assert_eq!(register_4_to_6_setting.operand2, 1.into());
        let register_3_to_6_setting = bb6
            .content
            .iter()
            .filter_map(|it| it.try_as_binary_calculate())
            .find(|it| it.to == RegisterName("_loop_3_4_1_goto_3_in_6".to_string()))
            .unwrap();
        assert_eq!(register_3_to_6_setting.operation, BinaryOperation::NotEqual);
        assert_eq!(register_3_to_6_setting.operand1, 0.into());
        assert_eq!(register_3_to_6_setting.operand2, 1.into());
        let dispatcher_0 = editor
            .content
            .content
            .iter()
            .find(|it| {
                it.name
                    .as_ref()
                    .map(|it| it == "_loop_3_4_1_dispatcher_at_7")
                    .unwrap_or(false)
            })
            .unwrap();
        let loop_3_4_1_dispatcher_at_7_var = dispatcher_0
            .content
            .iter()
            .filter_map(|it| it.try_as_phi())
            .find(|it| it.to == RegisterName("_loop_3_4_1_dispatcher_at_7_var".to_string()))
            .unwrap();
        let from_bb3 = loop_3_4_1_dispatcher_at_7_var
            .from
            .iter()
            .find(|it| it.block == "bb3")
            .unwrap();
        assert_eq!(
            from_bb3.value,
            RegisterName("_loop_3_4_1_goto_4_in_3".into()).into()
        );
        let from_bb4 = loop_3_4_1_dispatcher_at_7_var
            .from
            .iter()
            .find(|it| it.block == "bb4")
            .unwrap();
        assert_eq!(from_bb4.value, 0.into());
        let from_bb2 = loop_3_4_1_dispatcher_at_7_var
            .from
            .iter()
            .find(|it| it.block == "bb2")
            .unwrap();
        assert_eq!(from_bb2.value, 0.into());
        let dispatch_statement0 = dispatcher_0.content.last().unwrap().as_branch();
        assert_eq!(dispatch_statement0.branch_type, BranchType::NE);
        assert_eq!(
            dispatch_statement0.operand1,
            RegisterName("_loop_3_4_1_dispatcher_at_7_var".to_string()).into()
        );
        assert_eq!(dispatch_statement0.operand2, 0.into());
        assert_eq!(dispatch_statement0.success_label, "bb4");
        assert_eq!(
            dispatch_statement0.failure_label,
            "_loop_3_4_1_dispatcher_at_8"
        );
    }
}
