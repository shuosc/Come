use crate::ir::optimize::editor::IRFunctionEditor;

use super::IsPass;
pub struct RemoveLoadDirectlyAfterStore;

impl IsPass for RemoveLoadDirectlyAfterStore {
    fn run<'a>(&self, editor: &mut IRFunctionEditor) {
        let mut to_remove = Vec::new();
        let mut to_replace = Vec::new();
        for memory_access_info in editor.analyzer.memory_access_info().values() {
            let dorminate = memory_access_info.dorminate_in_basic_block();
            for (store, loads) in dorminate {
                let content = editor.content.borrow();
                let store_statement = content[store].as_store();
                let stored_value = store_statement.source.clone();
                for load in loads {
                    let load_statement = content[load.clone()].as_load();
                    to_replace.push((load_statement.to.clone(), stored_value.clone()));
                    to_remove.push(load);
                }
            }
        }
        to_remove.sort();
        for to_remove_index in to_remove.into_iter().rev() {
            editor.remove_statement(&to_remove_index);
        }
        for (register, value) in to_replace {
            editor.replace_register(&register, &value);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ir::{
            function::basic_block::BasicBlock,
            statement::{
                calculate::binary::BinaryOperation, Alloca, BinaryCalculate, IRStatement, Load,
                Store,
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
                    ],
                },
            ],
        };
        let pass = RemoveLoadDirectlyAfterStore;
        let mut editor = IRFunctionEditor::new(function);
        pass.run(&mut editor);
        let function = editor.done();
        println!("{}", function);
        assert_eq!(function.content[0].content.len(), 6);
        assert_eq!(
            function.content[0]
                .content
                .iter()
                .filter(|it| matches!(it, IRStatement::Load(_)))
                .count(),
            0
        );
        assert_eq!(function.content[1].content.len(), 2);
    }
}
