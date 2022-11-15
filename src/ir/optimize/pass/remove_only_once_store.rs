use crate::ir::{optimize::analyzer::MemoryAccessInfo, statement::IsIRStatement};

use super::IsPass;

pub struct RemoveOnlyOnceStore;

impl IsPass for RemoveOnlyOnceStore {
    fn run<'a>(&self, editor: &mut super::IRFunctionEditor) {
        let mut to_remove = Vec::new();
        let mut to_edit = Vec::new();
        for MemoryAccessInfo {
            alloca,
            store,
            load,
        } in editor.analyzer.memory_access_info().values()
        {
            if store.len() == 1 {
                // we are going to replace all useage of this memory address with the value
                // stored in, so we can remove the allocation and this store
                to_remove.push(alloca.clone());
                to_remove.push(store[0].clone());
                let value_stored_in = editor.content.borrow()[store[0].clone()]
                    .clone()
                    .as_store()
                    .source
                    .clone();
                // replace each load target with the value stored in
                for load_statement_index in load {
                    to_remove.push(load_statement_index.clone());
                    let load_result_register = editor.content.borrow()
                        [load_statement_index.clone()]
                    .generate_register()
                    .unwrap()
                    .0;
                    to_edit.push((load_result_register, value_stored_in.clone()));
                }
            }
        }
        to_remove.sort();
        for to_remove_index in to_remove.iter().rev() {
            editor.remove_statement(to_remove_index);
        }
        for (load_result_register, store_source) in to_edit {
            editor.replace_register(&load_result_register, &store_source);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{
        ir::{
            function::basic_block::BasicBlock,
            optimize::{pass::IsPass, IRFunctionEditor},
            statement::{
                calculate::binary::BinaryOperation, Alloca, BinaryCalculate, IsIRStatement, Jump,
                Load, Ret, Store,
            },
            FunctionDefinition, RegisterName,
        },
        utility::data_type::{self, Type},
    };

    use super::RemoveOnlyOnceStore;

    #[test]
    fn run() {
        let function = FunctionDefinition {
            name: "f".to_string(),
            parameters: Vec::new(),
            return_type: Type::None,
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
        let pass = RemoveOnlyOnceStore;
        let mut editor = IRFunctionEditor::new(function);
        pass.run(&mut editor);
        let function = editor.done();
        // %0 and %2 should be optimized out
        let mut registers = HashSet::new();
        for statement in function.iter() {
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
