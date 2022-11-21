use std::collections::{HashMap, HashSet};

use crate::ir::{
    optimize::editor::{EditActionsBatch, IRFunctionEditor},
    quantity::Quantity,
    statement::{IRStatement, Load, Store},
    RegisterName,
};

use super::IsPass;

pub struct MemoryToRegister;

fn insert_phi_positions(editor: &mut IRFunctionEditor) -> HashSet<(RegisterName, usize)> {
    let mut result = HashSet::new();
    let memory_access_info: Vec<_> = editor
        .analyzer
        .memory_access_info()
        .iter()
        .map(|(fst, snd)| (fst.clone(), snd.clone()))
        .collect();
    for (variable_name, memory_access_info) in memory_access_info {
        let stores_used_by_other = memory_access_info.stores_used_by_other_blocks();
        let mut pending_bb_indexes: Vec<_> = stores_used_by_other.iter().map(|it| it.0).collect();
        let mut done_bb_index = Vec::new();
        while !pending_bb_indexes.is_empty() {
            let considering_bb_index = pending_bb_indexes.pop().unwrap();
            done_bb_index.push(considering_bb_index);
            let dominator_frontier_bb_indexes = editor
                .analyzer
                .control_flow_graph()
                .dorminate_frontier(considering_bb_index);
            for to_bb_index in dominator_frontier_bb_indexes.clone() {
                result.insert((variable_name.clone(), to_bb_index));
                // it's possible we put a phi node to a new block which contains no
                // store to this variable in the past, in such cases we should look at the bacic block
                // of this phi node too
                let to_bb_had_no_store_to_this_variable = !done_bb_index.contains(&to_bb_index)
                    && !pending_bb_indexes.contains(&to_bb_index);
                if to_bb_had_no_store_to_this_variable {
                    #[cfg(test)]
                    cov_mark::hit!(generated_phi_spread_value);

                    pending_bb_indexes.push(to_bb_index);
                }
            }
        }
    }
    result
}

fn fill_first_block_name(editor: &mut IRFunctionEditor) {
    let function = editor.content();
    let function_name = function.name.clone();
    drop(function);
    editor
        .index_mut(0)
        .name
        .get_or_insert(format!("{}_entry", function_name));
}

fn variable_name_in_block(variable_name: &RegisterName, block_name: String) -> RegisterName {
    RegisterName(format!("{}_{}", variable_name.0, block_name))
}

fn decide_variable_value(
    variable_name: &RegisterName,
    current_variable_value: &[HashMap<RegisterName, (usize, Quantity)>],
) -> (usize, Quantity) {
    for frame in current_variable_value.iter().rev() {
        if let Some(value) = frame.get(variable_name) {
            return value.clone();
        }
    }
    unreachable!()
}

fn replace_load_start_from(
    editor: &IRFunctionEditor,
    from_block: Option<usize>,
    to_consider_block: usize,
    inserted_phi: &HashSet<(RegisterName, usize)>,
    visited: &mut HashSet<(Option<usize>, usize)>,
    current_variable_value: &mut Vec<HashMap<RegisterName, (usize, Quantity)>>,
) -> EditActionsBatch {
    let mut result = EditActionsBatch::default();
    if !visited.insert((from_block, to_consider_block)) {
        return result;
    }
    let block = editor.index(to_consider_block);
    let phied_variables = inserted_phi
        .iter()
        .filter(|(_, bb_id)| bb_id == &to_consider_block)
        .map(|(variable_name, _)| variable_name);
    for variable_name in phied_variables {
        let (from_basic_block_index, value) =
            decide_variable_value(variable_name, current_variable_value);
        result.insert_phi(
            variable_name.clone(),
            from_basic_block_index,
            to_consider_block,
            value,
        );
        current_variable_value.last_mut().unwrap().insert(
            variable_name.clone(),
            (
                to_consider_block,
                variable_name_in_block(variable_name, block.name.clone().unwrap()).into(),
            ),
        );
    }
    for (statement_index, statement) in block.content.iter().enumerate() {
        match statement {
            IRStatement::Load(Load {
                to,
                from: Quantity::RegisterName(local),
                ..
            }) => {
                let (_, replace_with_value) = decide_variable_value(local, current_variable_value);
                result.replace(to.clone(), replace_with_value);
                result.remove((to_consider_block, statement_index));
            }
            IRStatement::Store(Store {
                source,
                target: Quantity::RegisterName(local),
                ..
            }) => {
                current_variable_value
                    .last_mut()
                    .unwrap()
                    .insert(local.clone(), (to_consider_block, source.clone()));
                result.remove((to_consider_block, statement_index));
            }
            IRStatement::Branch(branch) => {
                let success_block = editor
                    .analyzer
                    .basic_block_index(&Some(branch.success_label.clone()));
                current_variable_value.push(HashMap::new());
                let result_success = replace_load_start_from(
                    editor,
                    Some(to_consider_block),
                    success_block,
                    inserted_phi,
                    visited,
                    current_variable_value,
                );
                result = result.then(result_success);
                current_variable_value.pop();
                current_variable_value.push(HashMap::new());
                let failure_block = editor
                    .analyzer
                    .basic_block_index(&Some(branch.failure_label.clone()));
                let result_failure = replace_load_start_from(
                    editor,
                    Some(to_consider_block),
                    failure_block,
                    inserted_phi,
                    visited,
                    current_variable_value,
                );
                result = result.then(result_failure);
                current_variable_value.pop();
            }
            IRStatement::Jump(jump) => {
                let jump_to_block = editor.analyzer.basic_block_index(&Some(jump.label.clone()));
                let result_jump_to = replace_load_start_from(
                    editor,
                    Some(to_consider_block),
                    jump_to_block,
                    inserted_phi,
                    visited,
                    current_variable_value,
                );
                result = result.then(result_jump_to);
            }
            _ => (),
        }
    }
    result
}

fn replace_loads(editor: &mut IRFunctionEditor, inserted_phi: &HashSet<(RegisterName, usize)>) {
    let mut visited = HashSet::new();
    let mut current_variable_value = vec![HashMap::new()];
    let result = replace_load_start_from(
        editor,
        None,
        0,
        inserted_phi,
        &mut visited,
        &mut current_variable_value,
    );
    editor.execute_batch(result);
}

impl IsPass for MemoryToRegister {
    fn run(&self, editor: &mut IRFunctionEditor) {
        fill_first_block_name(editor);
        let insert_phis_at = insert_phi_positions(editor);
        replace_loads(editor, &insert_phis_at);
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::borrow_interior_mutable_const)]

    use crate::{
        ir::{
            function::basic_block::BasicBlock,
            statement::{
                branch::BranchType, calculate::binary::BinaryOperation, phi::PhiSource, Alloca,
                BinaryCalculate, Branch, IRStatement, Jump, Load, Ret, Store,
            },
            FunctionDefinition,
        },
        utility::data_type,
    };

    use super::*;

    fn load(target: &str, id: usize) -> IRStatement {
        Load {
            to: RegisterName(format!("{}_{}", target, id)),
            data_type: data_type::I32.clone(),
            from: RegisterName(format!("{}_addr", target)).into(),
        }
        .into()
    }
    fn store(target: &str) -> IRStatement {
        Store {
            data_type: data_type::I32.clone(),
            source: 1.into(),
            target: RegisterName(format!("{}_addr", target)).into(),
        }
        .into()
    }
    fn store_reg(target: &str, reg: &str, reg_id: usize) -> IRStatement {
        Store {
            data_type: data_type::I32.clone(),
            source: RegisterName(format!("{}_{}", reg, reg_id)).into(),
            target: RegisterName(format!("{}_addr", target)).into(),
        }
        .into()
    }
    fn add(target: &str, src1: &str, src1_id: usize, src2: &str, src2_id: usize) -> IRStatement {
        BinaryCalculate {
            operation: BinaryOperation::Add,
            operand1: RegisterName(format!("{}_{}", src1, src1_id)).into(),
            operand2: RegisterName(format!("{}_{}", src2, src2_id)).into(),
            to: RegisterName(target.to_string()),
            data_type: data_type::I32.clone(),
        }
        .into()
    }
    fn alloca(target: &str) -> IRStatement {
        Alloca {
            to: RegisterName(format!("{}_addr", target)),
            alloc_type: data_type::I32.clone(),
        }
        .into()
    }
    fn jump(target: &str) -> IRStatement {
        Jump {
            label: target.to_string(),
        }
        .into()
    }
    fn br(target1: &str, target2: &str) -> IRStatement {
        Branch {
            branch_type: BranchType::EQ,
            operand1: 0.into(),
            operand2: 1.into(),
            success_label: target1.to_string(),
            failure_label: target2.to_string(),
        }
        .into()
    }

    #[test]
    fn simple() {
        let function_definition = FunctionDefinition {
            name: "f".to_string(),
            parameters: Vec::new(),
            return_type: data_type::I32.clone(),
            content: vec![
                BasicBlock {
                    name: None,
                    content: vec![
                        alloca("a"),
                        alloca("b"),
                        alloca("c"),
                        store("a"),
                        store("b"),
                        store("c"),
                        br("bb1", "bb2"),
                    ],
                },
                BasicBlock {
                    name: Some("bb1".to_string()),
                    content: vec![
                        load("a", 0),
                        load("b", 0),
                        add("t_0", "a", 0, "b", 0),
                        store_reg("c", "t", 0),
                        jump("bb3"),
                    ],
                },
                BasicBlock {
                    name: Some("bb2".to_string()),
                    content: vec![
                        load("a", 1),
                        load("b", 1),
                        add("t_1", "a", 1, "b", 1),
                        store_reg("c", "t", 1),
                        jump("bb3"),
                    ],
                },
                BasicBlock {
                    name: Some("bb3".to_string()),
                    content: vec![
                        load("c", 0),
                        Ret {
                            value: Some(RegisterName("c_0".to_string()).into()),
                        }
                        .into(),
                    ],
                },
            ],
        };
        let mut editor = IRFunctionEditor::new(function_definition);
        MemoryToRegister.run(&mut editor);
        let function_definition = editor.done();
        let the_phi_statement = function_definition.content[3].content[0].as_phi();
        assert_eq!(the_phi_statement.to, RegisterName("c_addr_bb3".to_string()));
        assert_eq!(the_phi_statement.from.len(), 2);
        assert!(the_phi_statement.from.contains(&PhiSource {
            name: RegisterName("t_0".to_string()).into(),
            block: "bb1".to_string()
        }));
        assert!(the_phi_statement.from.contains(&PhiSource {
            name: RegisterName("t_1".to_string()).into(),
            block: "bb2".to_string()
        }));
    }

    #[test]
    fn not_storing_unused() {
        let function_definition = FunctionDefinition {
            name: "f".to_string(),
            parameters: Vec::new(),
            return_type: data_type::I32.clone(),
            content: vec![
                BasicBlock {
                    name: None,
                    content: vec![
                        alloca("a"),
                        alloca("b"),
                        alloca("c"),
                        store("a"),
                        store("b"),
                        // we don't store c here
                        br("bb1", "bb2"),
                    ],
                },
                BasicBlock {
                    name: Some("bb1".to_string()),
                    content: vec![
                        load("a", 0),
                        load("b", 0),
                        add("t_0", "a", 0, "b", 0),
                        store_reg("c", "t", 0),
                        jump("bb3"),
                    ],
                },
                BasicBlock {
                    name: Some("bb2".to_string()),
                    content: vec![
                        load("a", 1),
                        load("b", 1),
                        add("t_1", "a", 1, "b", 1),
                        store_reg("c", "t", 1),
                        jump("bb3"),
                    ],
                },
                BasicBlock {
                    name: Some("bb3".to_string()),
                    content: vec![
                        load("c", 0),
                        Ret {
                            value: Some(RegisterName("c_0".to_string()).into()),
                        }
                        .into(),
                    ],
                },
            ],
        };
        let mut editor = IRFunctionEditor::new(function_definition);
        MemoryToRegister.run(&mut editor);
        let function_definition = editor.done();
        let the_phi_statement = function_definition.content[3].content[0].as_phi();
        assert_eq!(the_phi_statement.to, RegisterName("c_addr_bb3".to_string()));
        assert_eq!(the_phi_statement.from.len(), 2);
        assert!(the_phi_statement.from.contains(&PhiSource {
            name: RegisterName("t_0".to_string()).into(),
            block: "bb1".to_string()
        }));
        assert!(the_phi_statement.from.contains(&PhiSource {
            name: RegisterName("t_1".to_string()).into(),
            block: "bb2".to_string()
        }));
    }

    #[test]
    fn remove_load_in_multiple_basic_blocks() {
        let function_definition = FunctionDefinition {
            name: "f".to_string(),
            parameters: Vec::new(),
            return_type: data_type::I32.clone(),
            content: vec![
                BasicBlock {
                    name: None,
                    content: vec![
                        alloca("a"),
                        alloca("b"),
                        alloca("c"),
                        store("a"),
                        store("b"),
                        store("c"),
                        br("bb1", "bb2"),
                    ],
                },
                BasicBlock {
                    name: Some("bb1".to_string()),
                    content: vec![
                        load("a", 0),
                        load("b", 0),
                        add("t_0", "a", 0, "b", 0),
                        store_reg("c", "t", 0),
                        jump("bb4"),
                    ],
                },
                BasicBlock {
                    name: Some("bb2".to_string()),
                    content: vec![
                        load("a", 1),
                        load("b", 1),
                        add("t_1", "a", 1, "b", 1),
                        store_reg("c", "t", 1),
                        br("bb3", "bb4"),
                    ],
                },
                BasicBlock {
                    name: Some("bb3".to_string()),
                    content: vec![
                        load("a", 2),
                        load("b", 2),
                        load("c", 0),
                        add("t_2", "a", 2, "b", 2),
                        add("t_3", "t", 2, "c", 0),
                        store_reg("c", "t", 3),
                        jump("bb4"),
                    ],
                },
                BasicBlock {
                    name: Some("bb4".to_string()),
                    content: vec![
                        load("a", 3),
                        load("b", 3),
                        store("a"),
                        store("b"),
                        load("a", 4),
                        load("b", 4),
                        add("t_4", "a", 4, "b", 4),
                        load("c", 1),
                        add("t_5", "t", 4, "c", 1),
                        Ret {
                            value: Some(RegisterName("t_5".to_string()).into()),
                        }
                        .into(),
                    ],
                },
            ],
        };
        let mut editor = IRFunctionEditor::new(function_definition);
        MemoryToRegister.run(&mut editor);
        let function_definition = editor.done();
        let generated_phi = function_definition.content[4].content[0].as_phi();
        assert_eq!(generated_phi.to, RegisterName("c_addr_bb4".to_string()));
        assert_eq!(generated_phi.from.len(), 3);
        assert!(generated_phi.from.contains(&PhiSource {
            name: RegisterName("t_3".to_string()).into(),
            block: "bb3".to_string()
        }));
        assert!(generated_phi.from.contains(&PhiSource {
            name: RegisterName("t_1".to_string()).into(),
            block: "bb2".to_string()
        }));
        assert!(generated_phi.from.contains(&PhiSource {
            name: RegisterName("t_0".to_string()).into(),
            block: "bb1".to_string()
        }));
    }

    #[test]
    fn generate_store_like_phi() {
        let function_definition = FunctionDefinition {
            name: "f".to_string(),
            parameters: Vec::new(),
            return_type: data_type::I32.clone(),
            content: vec![
                BasicBlock {
                    name: None,
                    content: vec![alloca("a"), store("a"), br("bb2", "bb5")],
                },
                BasicBlock {
                    name: Some("bb2".to_string()),
                    content: vec![br("bb3", "bb4")],
                },
                BasicBlock {
                    name: Some("bb3".to_string()),
                    content: vec![load("a", 0), store("a"), jump("bb7")],
                },
                BasicBlock {
                    name: Some("bb4".to_string()),
                    content: vec![load("a", 1), store("a"), jump("bb7")],
                },
                BasicBlock {
                    name: Some("bb7".to_string()),
                    content: vec![load("a", 2), jump("bb6")],
                },
                BasicBlock {
                    name: Some("bb5".to_string()),
                    content: vec![
                        load("a", 3),
                        add("t_0", "a", 3, "a", 3),
                        store_reg("a", "t", 0),
                        jump("bb6"),
                    ],
                },
                BasicBlock {
                    name: Some("bb6".to_string()),
                    content: vec![
                        load("a", 4),
                        Ret {
                            value: Some(RegisterName("a_4".to_string()).into()),
                        }
                        .into(),
                    ],
                },
            ],
        };
        let mut editor = IRFunctionEditor::new(function_definition);
        cov_mark::check!(generated_phi_spread_value);
        MemoryToRegister.run(&mut editor);
        let function_definition = editor.done();
        let generated_phi = function_definition.content[4].content[0].as_phi();
        assert_eq!(generated_phi.to, RegisterName("a_addr_bb7".to_string()));
        assert_eq!(generated_phi.from.len(), 2);
        assert!(generated_phi.from.contains(&PhiSource {
            name: 1.into(),
            block: "bb4".to_string()
        }));
        assert!(generated_phi.from.contains(&PhiSource {
            name: 1.into(),
            block: "bb3".to_string()
        }));

        let generated_phi = function_definition.content[6].content[0].as_phi();
        assert_eq!(generated_phi.to, RegisterName("a_addr_bb6".to_string()));
        assert_eq!(generated_phi.from.len(), 2);
        assert!(generated_phi.from.contains(&PhiSource {
            name: RegisterName("t_0".to_string()).into(),
            block: "bb5".to_string()
        }));
        assert!(generated_phi.from.contains(&PhiSource {
            name: RegisterName("a_addr_bb7".to_string()).into(),
            block: "bb7".to_string()
        }));
    }

    #[test]
    fn self_refrence_phi() {
        let function_definition = FunctionDefinition {
            name: "f".to_string(),
            parameters: Vec::new(),
            return_type: data_type::I32.clone(),
            content: vec![
                BasicBlock {
                    name: None,
                    content: vec![alloca("a"), store("a"), br("bb2", "bb5")],
                },
                BasicBlock {
                    name: Some("bb2".to_string()),
                    content: vec![br("bb3", "bb4")],
                },
                BasicBlock {
                    name: Some("bb3".to_string()),
                    content: vec![load("a", 0), store("a"), jump("bb7")],
                },
                BasicBlock {
                    name: Some("bb4".to_string()),
                    content: vec![load("a", 1), store("a"), jump("bb7")],
                },
                BasicBlock {
                    name: Some("bb7".to_string()),
                    content: vec![load("a", 2), jump("bb6")],
                },
                BasicBlock {
                    name: Some("bb5".to_string()),
                    content: vec![
                        load("a", 3),
                        add("t_0", "a", 3, "a", 3),
                        store_reg("a", "t", 0),
                        br("bb6", "bb5"),
                    ],
                },
                BasicBlock {
                    name: Some("bb6".to_string()),
                    content: vec![
                        load("a", 4),
                        Ret {
                            value: Some(RegisterName("a_4".to_string()).into()),
                        }
                        .into(),
                    ],
                },
            ],
        };
        let mut editor = IRFunctionEditor::new(function_definition);
        MemoryToRegister.run(&mut editor);
        let function_definition = editor.done();

        let generated_phi = function_definition.content[4].content[0].as_phi();
        assert_eq!(generated_phi.to, RegisterName("a_addr_bb7".to_string()));
        assert_eq!(generated_phi.from.len(), 2);
        assert!(generated_phi.from.contains(&PhiSource {
            name: 1.into(),
            block: "bb3".to_string()
        }));
        assert!(generated_phi.from.contains(&PhiSource {
            name: 1.into(),
            block: "bb4".to_string()
        }));

        let generated_phi = function_definition.content[5].content[0].as_phi();
        assert_eq!(generated_phi.to, RegisterName("a_addr_bb5".to_string()));
        assert_eq!(generated_phi.from.len(), 2);
        assert!(generated_phi.from.contains(&PhiSource {
            name: 1.into(),
            block: "f_entry".to_string()
        }));
        assert!(generated_phi.from.contains(&PhiSource {
            name: RegisterName("t_0".to_string()).into(),
            block: "bb5".to_string()
        }));

        let generated_phi = function_definition.content[6].content[0].as_phi();
        assert_eq!(generated_phi.to, RegisterName("a_addr_bb6".to_string()));
        assert_eq!(generated_phi.from.len(), 2);
        assert!(generated_phi.from.contains(&PhiSource {
            name: RegisterName("a_addr_bb7".to_string()).into(),
            block: "bb7".to_string()
        }));
        assert!(generated_phi.from.contains(&PhiSource {
            name: RegisterName("t_0".to_string()).into(),
            block: "bb5".to_string()
        }));
    }

    #[test]
    fn comprehensive() {
        let function_definition = FunctionDefinition {
            name: "f".to_string(),
            parameters: Vec::new(),
            return_type: data_type::I32.clone(),
            content: vec![
                BasicBlock {
                    name: None,
                    content: vec![alloca("a"), store("a"), br("bb2", "bb4")],
                },
                BasicBlock {
                    name: Some("bb2".to_string()),
                    content: vec![
                        load("a", 0),
                        add("t_0", "a", 0, "a", 0),
                        store_reg("a", "t", 0),
                        jump("bb3"),
                    ],
                },
                BasicBlock {
                    name: Some("bb3".to_string()),
                    content: vec![
                        load("a", 1),
                        add("t_1", "a", 1, "a", 1),
                        store_reg("a", "t", 1),
                        br("bb2", "bb8"),
                    ],
                },
                BasicBlock {
                    name: Some("bb4".to_string()),
                    content: vec![br("bb5", "bb6")],
                },
                BasicBlock {
                    name: Some("bb5".to_string()),
                    content: vec![load("a", 2), store("a"), jump("bb7")],
                },
                BasicBlock {
                    name: Some("bb6".to_string()),
                    content: vec![load("a", 3), store("a"), jump("bb7")],
                },
                BasicBlock {
                    name: Some("bb7".to_string()),
                    content: vec![load("a", 4), jump("bb8")],
                },
                BasicBlock {
                    name: Some("bb8".to_string()),
                    content: vec![
                        load("a", 5),
                        Ret {
                            value: Some(RegisterName("a_5".to_string()).into()),
                        }
                        .into(),
                    ],
                },
            ],
        };
        let mut editor = IRFunctionEditor::new(function_definition);
        MemoryToRegister.run(&mut editor);
        let function_definition = editor.done();

        let generated_phi = function_definition.content[1].content[0].as_phi();
        assert_eq!(generated_phi.to, RegisterName("a_addr_bb2".to_string()));
        assert_eq!(generated_phi.from.len(), 2);
        assert!(generated_phi.from.contains(&PhiSource {
            name: 1.into(),
            block: "f_entry".to_string()
        }));
        assert!(generated_phi.from.contains(&PhiSource {
            name: RegisterName("t_1".to_string()).into(),
            block: "bb3".to_string()
        }));

        let generated_phi = function_definition.content[6].content[0].as_phi();
        assert_eq!(generated_phi.to, RegisterName("a_addr_bb7".to_string()));
        assert_eq!(generated_phi.from.len(), 2);
        assert!(generated_phi.from.contains(&PhiSource {
            name: 1.into(),
            block: "bb5".to_string()
        }));
        assert!(generated_phi.from.contains(&PhiSource {
            name: 1.into(),
            block: "bb6".to_string()
        }));

        let generated_phi = function_definition.content[7].content[0].as_phi();
        assert_eq!(generated_phi.to, RegisterName("a_addr_bb8".to_string()));
        assert_eq!(generated_phi.from.len(), 2);
        assert!(generated_phi.from.contains(&PhiSource {
            name: RegisterName("t_1".to_string()).into(),
            block: "bb3".to_string()
        }));
        assert!(generated_phi.from.contains(&PhiSource {
            name: RegisterName("a_addr_bb7".to_string()).into(),
            block: "bb7".to_string()
        }));
    }
}
