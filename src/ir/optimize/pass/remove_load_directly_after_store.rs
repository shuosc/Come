use crate::ir::optimize::{action::EditActionBatch, analyzer::Analyzer};

use super::IsPass;
pub struct RemoveLoadDirectlyAfterStore;

impl IsPass for RemoveLoadDirectlyAfterStore {
    fn run(&self, analyzer: &Analyzer) -> EditActionBatch {
        let mut result = EditActionBatch::default();
        let variables = analyzer.memory_usage.variables();
        for variable in variables {
            let memory_access_info = analyzer.memory_usage.memory_access_info(variable);
            for store_statement_index in &memory_access_info.store {
                let store_statement = analyzer.content[store_statement_index.clone()].as_store();
                let stored_value = store_statement.source.clone();
                for load_statement_index in
                    memory_access_info.loads_dorminated_by_store(store_statement_index)
                {
                    let load_statement = analyzer.content[load_statement_index.clone()].as_load();
                    result.replace(load_statement.to.clone(), stored_value.clone());
                    result.remove(load_statement_index);
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::borrow_interior_mutable_const)]

    use crate::{
        ir::{
            function::basic_block::BasicBlock,
            optimize::test_util::execute_pass,
            statement::{
                calculate::binary::BinaryOperation, Alloca, BinaryCalculate, IRStatement, Jump,
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
            name: "f".to_string(),
            parameters: Vec::new(),
            return_type: Type::None,
            content: vec![
                BasicBlock {
                    name: Some("bb0".to_string()),
                    content: vec![
                        Alloca {
                            to: RegisterName("0".to_string()),
                            alloc_type: data_type::I32.clone(),
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
                            target: RegisterName("0".to_string()).into(),
                        }
                        .into(),
                        Store {
                            data_type: data_type::I32.clone(),
                            source: 43.into(),
                            target: RegisterName("1".to_string()).into(),
                        }
                        .into(),
                        Load {
                            to: RegisterName("3".to_string()),
                            data_type: data_type::I32.clone(),
                            from: RegisterName("0".to_string()).into(),
                        }
                        .into(),
                        BinaryCalculate {
                            operation: BinaryOperation::Add,
                            operand1: RegisterName("3".to_string()).into(),
                            operand2: 44.into(),
                            to: RegisterName("4".to_string()),
                            data_type: data_type::I32.clone(),
                        }
                        .into(),
                        Load {
                            to: RegisterName("5".to_string()),
                            data_type: data_type::I32.clone(),
                            from: RegisterName("0".to_string()).into(),
                        }
                        .into(),
                        BinaryCalculate {
                            operation: BinaryOperation::Add,
                            operand1: RegisterName("5".to_string()).into(),
                            operand2: 45.into(),
                            to: RegisterName("6".to_string()),
                            data_type: data_type::I32.clone(),
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
                            operand2: 46.into(),
                            to: RegisterName("7".to_string()),
                            data_type: data_type::I32.clone(),
                        }
                        .into(),
                        Ret { value: None }.into(),
                    ],
                },
            ],
        };
        let pass = RemoveLoadDirectlyAfterStore;
        let function = execute_pass(function, pass.into());
        assert_eq!(function.content[0].content.len(), 7);
        assert_eq!(
            function.content[0]
                .content
                .iter()
                .filter(|it| matches!(it, IRStatement::Load(_)))
                .count(),
            0
        );
        assert_eq!(function.content[1].content.len(), 3);
    }
}
