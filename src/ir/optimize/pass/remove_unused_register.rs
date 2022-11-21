use std::collections::HashMap;

use crate::ir::statement::IsIRStatement;

use super::IsPass;

pub struct RemoveUnusedRegister;

impl IsPass for RemoveUnusedRegister {
    fn run(&self, editor: &mut super::IRFunctionEditor) {
        let content = editor.content();
        let mut registers: HashMap<_, _> = content
            .iter()
            .function_definition_index_enumerate()
            .filter_map(|(index, it)| {
                it.generate_register()
                    .map(|(register_name, _)| (register_name, index))
            })
            .collect();
        for statement in content.iter() {
            for register in statement.use_register() {
                registers.remove(&register);
            }
        }
        let mut to_remove: Vec<_> = registers.values().into_iter().collect();
        to_remove.sort();
        drop(content);
        for to_remove in to_remove.into_iter().rev() {
            editor.remove_statement(to_remove);
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::borrow_interior_mutable_const)]

    use super::*;
    use crate::{
        ir::{
            function::basic_block::BasicBlock,
            optimize::IRFunctionEditor,
            statement::{
                branch::BranchType, calculate::binary::BinaryOperation, BinaryCalculate, Branch,
                Jump, Load, Ret,
            },
            FunctionDefinition, RegisterName,
        },
        utility::data_type::{self, Type},
    };

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
                        Load {
                            to: RegisterName("0".to_string()),
                            data_type: data_type::I32.clone(),
                            from: 0.into(),
                        }
                        .into(),
                        Load {
                            to: RegisterName("1".to_string()),
                            data_type: data_type::I32.clone(),
                            from: 4.into(),
                        }
                        .into(),
                        Load {
                            to: RegisterName("3".to_string()),
                            data_type: data_type::I32.clone(),
                            from: 4.into(),
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
                        BinaryCalculate {
                            operation: BinaryOperation::Add,
                            operand1: RegisterName("0".to_string()).into(),
                            operand2: RegisterName("1".to_string()).into(),
                            to: RegisterName("2".to_string()),
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
                    content: vec![Branch {
                        branch_type: BranchType::NE,
                        operand1: RegisterName("2".to_string()).into(),
                        operand2: 0.into(),
                        success_label: "bb1".to_string(),
                        failure_label: "bb3".to_string(),
                    }
                    .into()],
                },
                BasicBlock {
                    name: Some("bb3".to_string()),
                    content: vec![Ret {
                        value: Some(RegisterName("2".to_string()).into()),
                    }
                    .into()],
                },
            ],
        };
        let pass = RemoveUnusedRegister;
        let mut editor = IRFunctionEditor::new(function);
        pass.run(&mut editor);
        let function = editor.done();
        assert_eq!(function.content[0].content.len(), 3);
    }
}
