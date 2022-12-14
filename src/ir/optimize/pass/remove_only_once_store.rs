use crate::ir::{analyzer::Analyzer, optimize::action::EditActionBatch};

use super::IsPass;

pub struct RemoveOnlyOnceStore;

impl IsPass for RemoveOnlyOnceStore {
    fn run(&self, analyzer: &Analyzer) -> EditActionBatch {
        let mut result = EditActionBatch::default();
        for variable in analyzer.memory_usage.variables() {
            let memory_access_info = analyzer.memory_usage.memory_access_info(variable);
            // todo: it is possible that the basic block the store statement in
            // cannot dorminate the block a load is in, in such cases, an error should
            // be raised instead of do this optimize work
            if memory_access_info.store.len() == 1 {
                let store_statement_index = memory_access_info.store[0].clone();
                let store_statement = analyzer.content[store_statement_index.clone()].as_store();
                let stored_value = store_statement.source.clone();
                for load_statement_index in &memory_access_info.load {
                    let load_statement = analyzer.content[load_statement_index.clone()].as_load();
                    result.replace(load_statement.to.clone(), stored_value.clone());
                    result.remove(load_statement_index.clone());
                }
                result.remove(store_statement_index.clone());
                result.remove(memory_access_info.alloca.clone());
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::borrow_interior_mutable_const)]

    use std::collections::HashSet;

    use crate::{
        ir::{
            function::basic_block::BasicBlock,
            optimize::test_util::execute_pass,
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
        let pass = RemoveOnlyOnceStore;
        let function = execute_pass(function, pass.into());
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
