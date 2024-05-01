use function::statement::{phi, BinaryCalculate, Ret};

use crate::{
    ir::{
        self,
        analyzer::{ControlFlowGraph, IsAnalyzer},
        function::{self, test_util::*},
        statement::branch,
        FunctionDefinition,
    },
    utility::data_type,
};

use super::*;

#[test]
fn test_generate_edit_plan() {
    let function_definition = FunctionDefinition {
        header: ir::FunctionHeader {
            name: "f".to_string(),
            parameters: Vec::new(),
            return_type: data_type::Type::None,
        },
        content: vec![
            jump_block(0, 1),
            jump_block(1, 2),
            branch_block(2, 3, 7),
            branch_block(3, 4, 8),
            branch_block(4, 6, 8),
            branch_block(5, 2, 10),
            branch_block(6, 2, 10),
            branch_block(7, 5, 9),
            branch_block(8, 3, 10),
            jump_block(9, 8),
            branch_block(10, 4, 6),
        ],
    };
    let control_flow_graph = ControlFlowGraph::new();
    let binded = control_flow_graph.bind(&function_definition);
    let mut origin_target_to_source_map = HashMap::new();
    origin_target_to_source_map.insert(
        binded.basic_block_index_by_name("bb3"),
        vec![
            IntoSccEdgeSource::One(binded.basic_block_index_by_name("bb2")),
            IntoSccEdgeSource::Two {
                source_block_index: binded.basic_block_index_by_name("bb8"),
                on_success: binded.basic_block_index_by_name("bb3"),
                on_failure: binded.basic_block_index_by_name("bb10"),
            },
        ],
    );
    origin_target_to_source_map.insert(
        binded.basic_block_index_by_name("bb8"),
        vec![
            IntoSccEdgeSource::One(binded.basic_block_index_by_name("bb3")),
            IntoSccEdgeSource::One(binded.basic_block_index_by_name("bb4")),
            IntoSccEdgeSource::One(binded.basic_block_index_by_name("bb9")),
        ],
    );
    origin_target_to_source_map.insert(
        binded.basic_block_index_by_name("bb10"),
        vec![
            IntoSccEdgeSource::Two {
                source_block_index: binded.basic_block_index_by_name("bb8"),
                on_success: binded.basic_block_index_by_name("bb8"),
                on_failure: binded.basic_block_index_by_name("bb10"),
            },
            IntoSccEdgeSource::One(binded.basic_block_index_by_name("bb5")),
            IntoSccEdgeSource::One(binded.basic_block_index_by_name("bb6")),
        ],
    );
    let plan = generate_edit_plan(&origin_target_to_source_map, &binded);
    assert_eq!(plan.phis.len(), 2);
    let phi_for_bb3 = plan
        .phis
        .iter()
        .find(|it| it.to == RegisterName("_should_goto_scc_3_8_10_bb3".to_string()))
        .unwrap();
    assert_eq!("%_should_goto_scc_3_8_10_bb3 = phi u1 [bb2, 1], [bb3, 0], [bb4, 0], [bb5, 0], [bb6, 0], [bb8, %_extracted_branch_condition_scc_3_8_10_at_bb8], [bb9, 0]", format!("{}", phi_for_bb3));
    let phi_for_bb8 = plan
        .phis
        .iter()
        .find(|it| it.to == RegisterName("_should_goto_scc_3_8_10_bb8".to_string()))
        .unwrap();
    assert_eq!("%_should_goto_scc_3_8_10_bb8 = phi u1 [bb2, 0], [bb3, 1], [bb4, 1], [bb5, 0], [bb6, 0], [bb8, 0], [bb9, 1]", format!("{}", phi_for_bb8));
    assert_eq!(plan.branches.len(), 2);
    let branch_for_bb3 = plan
        .branches
        .iter()
        .find(|it| it.success_label == "bb3")
        .unwrap();
    assert_eq!(
        "bne %_should_goto_scc_3_8_10_bb3, 0, bb3, _guard_block_scc_3_8_10_for_bb8",
        format!("{}", branch_for_bb3)
    );
    let branch_for_bb8 = plan
        .branches
        .iter()
        .find(|it| it.success_label == "bb8")
        .unwrap();
    assert_eq!(
        "bne %_should_goto_scc_3_8_10_bb8, 0, bb8, bb10",
        format!("{}", branch_for_bb8)
    );
    assert_eq!(plan.fix_other_block_plan.len(), 7);
    assert!(plan
        .fix_other_block_plan
        .contains(&FixOtherBlockPlan::DirectReplace {
            block: binded.basic_block_index_by_name("bb2"),
            origin_target: "bb3".to_string(),
        }));
    assert!(plan
        .fix_other_block_plan
        .contains(&FixOtherBlockPlan::ExtractCondition {
            block: binded.basic_block_index_by_name("bb8"),
            inverse: false
        }));
}

#[test]
fn test_inverse_condition() {
    let function_definition = FunctionDefinition {
        header: ir::FunctionHeader {
            name: "f".to_string(),
            parameters: Vec::new(),
            return_type: data_type::Type::None,
        },
        content: vec![
            branch_block(0, 1, 2),
            branch_block(1, 4, 3),
            branch_block(2, 3, 4),
            jump_block(3, 4),
            jump_block(4, 5),
            branch_block(5, 3, 6),
            ret_block(6),
        ],
    };
    let control_flow_graph = ControlFlowGraph::new();
    let binded = control_flow_graph.bind(&function_definition);
    let mut origin_target_to_source_map = HashMap::new();
    origin_target_to_source_map.insert(
        binded.basic_block_index_by_name("bb3"),
        vec![
            IntoSccEdgeSource::Two {
                source_block_index: binded.basic_block_index_by_name("bb1"),
                on_success: binded.basic_block_index_by_name("bb4"),
                on_failure: binded.basic_block_index_by_name("bb3"),
            },
            IntoSccEdgeSource::Two {
                source_block_index: binded.basic_block_index_by_name("bb2"),
                on_success: binded.basic_block_index_by_name("bb3"),
                on_failure: binded.basic_block_index_by_name("bb4"),
            },
        ],
    );
    origin_target_to_source_map.insert(
        binded.basic_block_index_by_name("bb4"),
        vec![
            IntoSccEdgeSource::Two {
                source_block_index: binded.basic_block_index_by_name("bb1"),
                on_success: binded.basic_block_index_by_name("bb4"),
                on_failure: binded.basic_block_index_by_name("bb3"),
            },
            IntoSccEdgeSource::Two {
                source_block_index: binded.basic_block_index_by_name("bb2"),
                on_success: binded.basic_block_index_by_name("bb3"),
                on_failure: binded.basic_block_index_by_name("bb4"),
            },
            IntoSccEdgeSource::One(binded.basic_block_index_by_name("bb3")),
        ],
    );
    let plan = generate_edit_plan(&origin_target_to_source_map, &binded);
    assert_eq!(plan.fix_other_block_plan.len(), 3);
    assert!(plan
        .fix_other_block_plan
        .contains(&FixOtherBlockPlan::ExtractCondition {
            block: binded.basic_block_index_by_name("bb1"),
            inverse: true
        }));
    assert!(plan
        .fix_other_block_plan
        .contains(&FixOtherBlockPlan::ExtractCondition {
            block: binded.basic_block_index_by_name("bb2"),
            inverse: false
        }));
}

#[test]
fn test_execute_edit_plan() {
    let mut function_definition = FunctionDefinition {
        header: ir::FunctionHeader {
            name: "f".to_string(),
            parameters: Vec::new(),
            return_type: data_type::Type::None,
        },
        content: vec![
            branch_block(0, 1, 3),
            jump_block(1, 2),
            jump_block(2, 3),
            jump_block(3, 1),
        ],
    };
    let edit_plan = EditPlan {
        scc_id: "1_3".to_string(),
        phis: vec![phi::parse(
            "%_should_goto_scc_1_3_bb1 = phi u1 [%_extracted_branch_condition_scc_1_3_at_bb0, bb0], [1, bb3]",
        ).unwrap().1],
        branches: vec![branch::parse(
            "bne %_should_goto_scc_1_3_bb1, 0, bb1, bb3"
        ).unwrap().1],
        fix_other_block_plan: [
            FixOtherBlockPlan::ExtractCondition { block: 0, inverse: false },
            FixOtherBlockPlan::DirectReplace { block: 2, origin_target: "bb3".to_string() },
            FixOtherBlockPlan::DirectReplace { block: 3, origin_target: "bb1".to_string() }
        ].into_iter().collect(),
    };
    execute_edit_plan(&mut function_definition, edit_plan);
    let bb0 = &function_definition.content[0];
    let condition_statement = bb0.content.iter().rev().nth(1).unwrap();
    let jump_statement = bb0.content.iter().last().unwrap();
    if let IRStatement::BinaryCalculate(BinaryCalculate { to, .. }) = condition_statement {
        assert_eq!(
            to,
            &RegisterName("_extracted_branch_condition_scc_1_3_at_bb0".to_string())
        )
    } else {
        panic!("condition_statement should be a `BinaryCalculate`");
    }
    if let IRStatement::Jump(Jump { label }) = jump_statement {
        assert_eq!(label, "_guard_block_scc_1_3_for_bb1");
    } else {
        panic!("jump_statement should be a `Jump`")
    }
    let bb2 = &function_definition.content[2];
    assert_eq!(
        bb2.content.last().unwrap().as_jump().label,
        "_guard_block_scc_1_3_for_bb1"
    );
    let bb3 = &function_definition.content[3];
    assert_eq!(
        bb3.content.last().unwrap().as_jump().label,
        "_guard_block_scc_1_3_for_bb1"
    );
    let new_block = &function_definition.content[4];
    assert_eq!(
        new_block.name.as_ref().unwrap(),
        "_guard_block_scc_1_3_for_bb1"
    );
    let phi = new_block.content[0].as_phi();
    assert_eq!(phi.to, RegisterName("_should_goto_scc_1_3_bb1".to_string()));
    assert_eq!(phi.from.len(), 2);
    assert!(phi.from.contains(&PhiSource {
        value: RegisterName("_extracted_branch_condition_scc_1_3_at_bb0".to_string()).into(),
        block: "bb0".to_string()
    }));
    assert!(phi.from.contains(&PhiSource {
        value: 1.into(),
        block: "bb3".to_string()
    }));
    let branch = new_block.content[1].as_branch();
    assert_eq!(
        branch.operand1,
        RegisterName("_should_goto_scc_1_3_bb1".to_string()).into()
    );
    assert_eq!(branch.operand2, 0.into());
    assert_eq!(branch.success_label, "bb1");
    assert_eq!(branch.failure_label, "bb3");
}

#[test]
fn test_generate_origin_target_to_source_map() {
    let function_definition = FunctionDefinition {
        header: ir::FunctionHeader {
            name: "f".to_string(),
            parameters: Vec::new(),
            return_type: data_type::Type::None,
        },
        content: vec![
            branch_block(0, 1, 3),
            jump_block(1, 2),
            jump_block(2, 3),
            jump_block(3, 1),
        ],
    };
    let edges_into_entry_nodes = vec![(0, 1), (0, 3), (3, 1), (2, 3)];
    let target_to_source_map =
        generate_origin_target_to_source_map(&function_definition, edges_into_entry_nodes);
    assert!(target_to_source_map
        .get(&1)
        .unwrap()
        .contains(&IntoSccEdgeSource::One(3)));
    assert!(target_to_source_map
        .get(&1)
        .unwrap()
        .contains(&IntoSccEdgeSource::Two {
            source_block_index: 0,
            on_success: 1,
            on_failure: 3
        }));
    assert!(target_to_source_map
        .get(&3)
        .unwrap()
        .contains(&IntoSccEdgeSource::One(2)));
    assert!(target_to_source_map
        .get(&3)
        .unwrap()
        .contains(&IntoSccEdgeSource::Two {
            source_block_index: 0,
            on_success: 1,
            on_failure: 3
        }));

    let function_definition = FunctionDefinition {
        header: ir::FunctionHeader {
            name: "f".to_string(),
            parameters: Vec::new(),
            return_type: data_type::Type::None,
        },
        content: vec![
            branch_block(0, 1, 3),
            jump_block(1, 2),
            jump_block(2, 3),
            branch_block(3, 1, 3),
        ],
    };
    let edges_into_entry_nodes = vec![(0, 1), (0, 3), (3, 3), (3, 1), (2, 3)];
    let target_to_source_map =
        generate_origin_target_to_source_map(&function_definition, edges_into_entry_nodes);
    assert!(target_to_source_map
        .get(&1)
        .unwrap()
        .contains(&IntoSccEdgeSource::Two {
            source_block_index: 3,
            on_success: 1,
            on_failure: 3
        }));
    assert!(target_to_source_map
        .get(&1)
        .unwrap()
        .contains(&IntoSccEdgeSource::Two {
            source_block_index: 0,
            on_success: 1,
            on_failure: 3
        }))
}

#[test]
fn test_fix_irreducible() {
    let function_definition = FunctionDefinition {
        header: ir::FunctionHeader {
            name: "f".to_string(),
            parameters: Vec::new(),
            return_type: data_type::Type::None,
        },
        content: vec![
            branch_block(0, 1, 2),
            branch_block(1, 3, 5),
            branch_block(3, 2, 9),
            jump_block(2, 1),
            branch_block(5, 4, 8),
            jump_block(4, 6),
            branch_block(8, 6, 3),
            jump_block(6, 7),
            jump_block(7, 8),
            ret_block(9),
        ],
    };
    let mut editor = Editor::new(function_definition);
    let pass = FixIrreducible;
    pass.run(&mut editor);
    assert_eq!(editor.content.content.len(), 12);
    let guard1 = editor
        .content
        .content
        .iter()
        .find(|it| it.name.as_ref().unwrap() == "_guard_block_scc_1_3_for_bb1")
        .unwrap();
    assert!(guard1.content.len() == 2);
    assert!(guard1.content[0].as_phi().from.contains(&PhiSource {
        value: RegisterName("_extracted_branch_condition_scc_1_3_at_bb0".to_string()).into(),
        block: "bb0".to_string()
    }));
    assert_eq!(
        guard1.content[1].as_branch().operand1,
        RegisterName("_should_goto_scc_1_3_bb1".to_string()).into()
    );

    let function_definition = FunctionDefinition {
        header: ir::FunctionHeader {
            name: "f".to_string(),
            parameters: Vec::new(),
            return_type: data_type::Type::None,
        },
        content: vec![
            jump_block(0, 1),
            jump_block(1, 2),
            branch_block(2, 3, 7),
            branch_block(3, 4, 8),
            branch_block(7, 5, 9),
            branch_block(4, 6, 8),
            branch_block(8, 3, 9),
            branch_block(6, 2, 10),
            branch_block(10, 6, 4),
            branch_block(5, 2, 10),
            jump_block(9, 8),
        ],
    };
    let mut editor = Editor::new(function_definition);
    let pass = FixIrreducible;
    pass.run(&mut editor);
    assert_eq!(editor.content.content.len(), 13);
    let guard1 = editor
        .content
        .content
        .iter()
        .find(|it| it.name.as_ref().unwrap() == "_guard_block_scc_3_8_10_for_bb3")
        .unwrap();
    assert!(guard1.content.len() == 3);
    assert!(guard1.content[0].as_phi().from.contains(&PhiSource {
        value: RegisterName("_extracted_branch_condition_scc_3_8_10_at_bb8".to_string()).into(),
        block: "bb8".to_string()
    }));
    assert_eq!(
        guard1.content[1].as_phi().to,
        RegisterName("_should_goto_scc_3_8_10_bb10".to_string())
    );

    let function_definition = FunctionDefinition {
        header: ir::FunctionHeader {
            name: "f".to_string(),
            parameters: Vec::new(),
            return_type: data_type::Type::None,
        },
        content: vec![
            jump_block(0, 1),
            jump_block(1, 2),
            branch_block(2, 3, 7),
            branch_block(3, 4, 8),
            branch_block(4, 6, 8),
            branch_block(8, 3, 10),
            branch_block(6, 2, 10),
            branch_block(10, 6, 4),
            branch_block(7, 5, 9),
            branch_block(5, 2, 10),
            jump_block(9, 8),
        ],
    };
    let mut editor = Editor::new(function_definition);
    let pass = FixIrreducible;
    pass.run(&mut editor);
    assert_eq!(editor.content.content.len(), 13);
    let guard1 = editor
        .content
        .content
        .iter()
        .find(|it| it.name.as_ref().unwrap() == "_guard_block_scc_3_5_7_for_bb3")
        .unwrap();
    assert!(guard1.content.len() == 3);
    assert!(guard1.content[0].as_phi().from.contains(&PhiSource {
        value: RegisterName("_extracted_branch_condition_scc_3_5_7_at_bb8".to_string()).into(),
        block: "bb8".to_string()
    }));
    assert_eq!(
        guard1.content[2].as_branch().failure_label,
        "_guard_block_scc_3_5_7_for_bb8"
    );
    let bb8 = editor
        .content
        .content
        .iter()
        .find(|it| it.name.as_ref().unwrap() == "bb8")
        .unwrap();
    assert_eq!(bb8.content.len(), 2);
    assert_eq!(
        bb8.content[0].as_binary_calculate().to,
        RegisterName("_extracted_branch_condition_scc_3_5_7_at_bb8".to_string())
    );
    assert_eq!(
        bb8.content[1].as_jump().label,
        "_guard_block_scc_3_5_7_for_bb3"
    );
}

#[test]
fn test_shit() {
    let function_definition = FunctionDefinition {
        header: ir::FunctionHeader {
            name: "f".to_string(),
            parameters: Vec::new(),
            return_type: data_type::Type::None,
        },
        content: vec![
            jump_block(0, 1),
            branch_block(1, 2, 3),
            branch_block(2, 3, 1),
            branch_block(3, 1, 2),
        ],
    };
    println!("{}", function_definition);
    let mut editor = Editor::new(function_definition);
    let pass = FixIrreducible;
    pass.run(&mut editor);
    println!("{}", editor.content);
}
