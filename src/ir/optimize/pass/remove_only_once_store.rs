use super::IsPass;

/// This pass will
/// - remove all store statements which is the only one store to a variable
/// - remove the load statements to the variable
/// - replace all usage of the load results to the source of the store
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct RemoveOnlyOnceStore;

impl IsPass for RemoveOnlyOnceStore {
    // fn run(&self, analyzer: &Analyzer) -> Actions {
    //     let mut result = Actions::default();
    //     for variable in analyzer.memory_usage.memory_access_variables() {
    //         let memory_access_info = analyzer.memory_usage.memory_access_info(variable);
    //         // todo: it is possible that the basic block the store statement in
    //         // cannot dorminate the block a load is in, in such cases, an error should
    //         // be raised instead of do this optimize work
    //         if memory_access_info.store.len() == 1 {
    //             let store_statement_index = memory_access_info.store[0].clone();
    //             let store_statement = analyzer.content[store_statement_index.clone()].as_store();
    //             let stored_value = store_statement.source.clone();
    //             for load_statement_index in &memory_access_info.load {
    //                 let load_statement = analyzer.content[load_statement_index.clone()].as_load();
    //                 result.push(RemoveStatement::new(load_statement_index.clone()));
    //                 result.push(RenameLocal::new(
    //                     load_statement.to.clone(),
    //                     stored_value.clone(),
    //                 ));
    //             }
    //             result.push(RemoveStatement::new(store_statement_index.clone()));
    //             result.push(RemoveStatement::new(memory_access_info.alloca.clone()));
    //         }
    //     }
    //     result
    // }

    fn need(&self) -> Vec<super::Pass> {
        Vec::new()
    }

    fn invalidate(&self) -> Vec<super::Pass> {
        Vec::new()
    }

    fn run(&self, editor: &mut crate::ir::editor::Editor) {
        let mut to_remove = Vec::new();
        let mut to_rename = Vec::new();
        for variable in editor
            .analyzer
            .memory_usage
            .memory_access_variables(&editor.content)
        {
            let memory_access_info = editor
                .analyzer
                .memory_usage
                .memory_access_info(&editor.content, variable);
            // todo: it is possible that the basic block the store statement in
            // cannot dominate the block a load is in, in such cases, an error should
            // be raised instead of do this optimize work
            if memory_access_info.store.len() == 1 {
                let store_statement_index = memory_access_info.store[0].clone();
                let store_statement = editor.content[store_statement_index.clone()].as_store();
                let stored_value = store_statement.source.clone();
                for load_statement_index in &memory_access_info.load {
                    let load_statement = editor.content[load_statement_index.clone()].as_load();
                    to_remove.push(load_statement_index.clone());
                    to_rename.push((load_statement.to.clone(), stored_value.clone()));
                }
                to_remove.push(store_statement_index.clone());
                to_remove.push(memory_access_info.alloca.clone().unwrap());
            }
        }
        editor.remove_statements(to_remove);
        for (from, to) in to_rename {
            editor.rename_local(from, to);
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::borrow_interior_mutable_const)]

    use std::collections::HashSet;

    use crate::{
        ir::{
            self,
            editor::Editor,
            function::basic_block::BasicBlock,
            optimize::pass::IsPass,
            statement::{
                calculate::binary::BinaryOperation, Alloca, BinaryCalculate, IsIRStatement, Jump,
                Load, Ret, Store,
            },
            FunctionDefinition, RegisterName,
        },
        utility::data_type::{self, Type},
    };

    use super::*;

    #[test]
    fn run() {
        let function = FunctionDefinition {
            header: ir::FunctionHeader {
                name: "f".to_string(),
                parameters: Vec::new(),
                return_type: Type::None,
            },
            content: vec![
                BasicBlock {
                    name: None,
                    content: vec![
                        Alloca {
                            to: RegisterName("0".to_string()),
                            alloc_type: data_type::I32.clone(),
                        }
                        .into(),
                        Store {
                            data_type: data_type::I32.clone(),
                            source: 42.into(),
                            target: RegisterName("0".to_string()).into(),
                        }
                        .into(),
                        Alloca {
                            to: RegisterName("1".to_string()),
                            alloc_type: data_type::I32.clone(),
                        }
                        .into(),
                        Store {
                            data_type: data_type::I32.clone(),
                            source: 42.into(),
                            target: RegisterName("1".to_string()).into(),
                        }
                        .into(),
                        Jump {
                            label: "bb1".to_string(),
                        }
                        .into(),
                    ],
                },
                BasicBlock {
                    name: Some("bb1".to_string()),
                    content: vec![
                        Load {
                            to: RegisterName("2".to_string()),
                            data_type: data_type::I32.clone(),
                            from: RegisterName("0".to_string()).into(),
                        }
                        .into(),
                        BinaryCalculate {
                            operation: BinaryOperation::Add,
                            operand1: RegisterName("2".to_string()).into(),
                            operand2: RegisterName("2".to_string()).into(),
                            to: RegisterName("3".to_string()),
                            data_type: data_type::I32.clone(),
                        }
                        .into(),
                        Load {
                            to: RegisterName("4".to_string()),
                            data_type: data_type::I32.clone(),
                            from: RegisterName("1".to_string()).into(),
                        }
                        .into(),
                        BinaryCalculate {
                            operation: BinaryOperation::Add,
                            operand1: RegisterName("4".to_string()).into(),
                            operand2: RegisterName("2".to_string()).into(),
                            to: RegisterName("5".to_string()),
                            data_type: data_type::I32.clone(),
                        }
                        .into(),
                        Jump {
                            label: "bb2".to_string(),
                        }
                        .into(),
                    ],
                },
                BasicBlock {
                    name: Some("bb2".to_string()),
                    content: vec![
                        Store {
                            data_type: data_type::I32.clone(),
                            source: 43.into(),
                            target: RegisterName("1".to_string()).into(),
                        }
                        .into(),
                        Load {
                            to: RegisterName("6".to_string()),
                            data_type: data_type::I32.clone(),
                            from: RegisterName("1".to_string()).into(),
                        }
                        .into(),
                        BinaryCalculate {
                            operation: BinaryOperation::Add,
                            operand1: RegisterName("6".to_string()).into(),
                            operand2: RegisterName("6".to_string()).into(),
                            to: RegisterName("7".to_string()),
                            data_type: data_type::I32.clone(),
                        }
                        .into(),
                        Jump {
                            label: "bb3".to_string(),
                        }
                        .into(),
                    ],
                },
                BasicBlock {
                    name: Some("bb3".to_string()),
                    content: vec![Ret {
                        value: Some(RegisterName("7".to_string()).into()),
                    }
                    .into()],
                },
            ],
        };
        let mut editor = Editor::new(function);
        let pass = RemoveOnlyOnceStore;
        pass.run(&mut editor);
        // %0 and %2 should be optimized out
        let mut registers = HashSet::new();
        for statement in editor.content.iter() {
            if let Some((r, _)) = statement.generate_register() {
                registers.insert(r);
            }
            registers.extend(statement.use_register());
        }
        assert!(!registers.contains(&RegisterName("0".to_string())));
        assert!(registers.contains(&RegisterName("1".to_string())));
        assert!(!registers.contains(&RegisterName("2".to_string())));
        assert!(registers.contains(&RegisterName("3".to_string())));
        assert!(registers.contains(&RegisterName("4".to_string())));
        assert!(registers.contains(&RegisterName("5".to_string())));
        assert!(registers.contains(&RegisterName("6".to_string())));
        assert!(registers.contains(&RegisterName("7".to_string())));
    }
}
